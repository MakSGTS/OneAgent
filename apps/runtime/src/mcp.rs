//! Bounded asynchronous MCP stream transport.

use std::error::Error;
use std::fmt;
use std::future::Future;

use oneagent_protocol::{EncodeError, MAX_MESSAGE_BYTES, McpServer, Response, encode_response};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const READ_SCRATCH_BYTES: usize = 8_192;

/// Successful terminal outcomes for an MCP stdio-compatible stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpStdioOutcome {
    /// The peer closed input between complete frames.
    EndOfInput,
    /// The injected shutdown source requested cancellation.
    Cancelled,
}

/// Closed terminal failure categories for the MCP stream adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpStdioErrorKind {
    /// Reading the input stream failed.
    Read,
    /// A frame was not valid UTF-8.
    InvalidUtf8,
    /// A frame exceeded the accepted payload bound.
    FrameTooLarge,
    /// EOF arrived after a non-empty unterminated frame.
    IncompleteFrame,
    /// A closed protocol response could not be encoded.
    Encode,
    /// Writing protocol output failed.
    Write,
    /// Flushing protocol output failed.
    Flush,
    /// The injected shutdown source failed.
    Shutdown,
}

impl McpStdioErrorKind {
    /// Returns the stable diagnostic category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read failure",
            Self::InvalidUtf8 => "invalid UTF-8 frame",
            Self::FrameTooLarge => "frame too large",
            Self::IncompleteFrame => "incomplete frame",
            Self::Encode => "response encoding failure",
            Self::Write => "write failure",
            Self::Flush => "flush failure",
            Self::Shutdown => "shutdown source failure",
        }
    }
}

/// A redacted terminal MCP stream failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpStdioError {
    kind: McpStdioErrorKind,
}

impl McpStdioError {
    const fn new(kind: McpStdioErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> McpStdioErrorKind {
        self.kind
    }
}

impl fmt::Display for McpStdioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind.as_str())
    }
}

impl Error for McpStdioError {}

/// A stateless newline-framed adapter around a transport-independent server.
#[derive(Debug, Default)]
pub struct McpStdioTransport {
    server: McpServer,
}

impl McpStdioTransport {
    /// Creates an adapter around the supplied server.
    #[must_use]
    pub const fn new(server: McpServer) -> Self {
        Self { server }
    }

    /// Runs sequential framing until EOF, cancellation, or one terminal error.
    ///
    /// The writer receives only compact JSON-RPC responses followed by LF.
    /// Notifications produce no output. This method starts no task.
    ///
    /// # Errors
    ///
    /// Returns a closed [`McpStdioError`] for framing, I/O, encoding, or
    /// shutdown-source failure.
    pub async fn run<R, W, F, E>(
        &self,
        reader: &mut R,
        writer: &mut W,
        shutdown: F,
    ) -> Result<McpStdioOutcome, McpStdioError>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
        F: Future<Output = Result<(), E>>,
        E: Error,
    {
        tokio::pin!(shutdown);
        let mut frame = Vec::new();
        let mut scratch = [0_u8; READ_SCRATCH_BYTES];

        loop {
            let read = tokio::select! {
                biased;
                outcome = &mut shutdown => return shutdown_outcome(outcome),
                outcome = reader.read(&mut scratch) => {
                    outcome.map_err(|_| McpStdioError::new(McpStdioErrorKind::Read))?
                }
            };

            if read == 0 {
                return if frame.is_empty() {
                    Ok(McpStdioOutcome::EndOfInput)
                } else {
                    Err(McpStdioError::new(McpStdioErrorKind::IncompleteFrame))
                };
            }

            for byte in &scratch[..read] {
                if *byte == b'\n' {
                    if frame.last() == Some(&b'\r') {
                        frame.pop();
                    }
                    if frame.len() > MAX_MESSAGE_BYTES {
                        return Err(McpStdioError::new(McpStdioErrorKind::FrameTooLarge));
                    }
                    self.process_frame(&frame, writer).await?;
                    frame.clear();
                    if poll_shutdown(&mut shutdown).await? {
                        return Ok(McpStdioOutcome::Cancelled);
                    }
                } else {
                    frame.push(*byte);
                    if frame.len() > MAX_MESSAGE_BYTES
                        && !(frame.len() == MAX_MESSAGE_BYTES + 1 && *byte == b'\r')
                    {
                        return Err(McpStdioError::new(McpStdioErrorKind::FrameTooLarge));
                    }
                }
            }
        }
    }

    async fn process_frame<W>(&self, frame: &[u8], writer: &mut W) -> Result<(), McpStdioError>
    where
        W: AsyncWrite + Unpin,
    {
        self.process_frame_with_encoder(frame, writer, encode_response)
            .await
    }

    async fn process_frame_with_encoder<W, E>(
        &self,
        frame: &[u8],
        writer: &mut W,
        encode: E,
    ) -> Result<(), McpStdioError>
    where
        W: AsyncWrite + Unpin,
        E: FnOnce(&Response) -> Result<Vec<u8>, EncodeError>,
    {
        let input = std::str::from_utf8(frame)
            .map_err(|_| McpStdioError::new(McpStdioErrorKind::InvalidUtf8))?;
        let Some(response) = self.server.dispatch(input) else {
            return Ok(());
        };
        let payload =
            encode(&response).map_err(|_| McpStdioError::new(McpStdioErrorKind::Encode))?;
        writer
            .write_all(&payload)
            .await
            .map_err(|_| McpStdioError::new(McpStdioErrorKind::Write))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|_| McpStdioError::new(McpStdioErrorKind::Write))?;
        writer
            .flush()
            .await
            .map_err(|_| McpStdioError::new(McpStdioErrorKind::Flush))?;
        Ok(())
    }
}

fn shutdown_outcome<E>(outcome: Result<(), E>) -> Result<McpStdioOutcome, McpStdioError> {
    outcome.map_or_else(
        |_| Err(McpStdioError::new(McpStdioErrorKind::Shutdown)),
        |()| Ok(McpStdioOutcome::Cancelled),
    )
}

async fn poll_shutdown<F, E>(shutdown: &mut std::pin::Pin<&mut F>) -> Result<bool, McpStdioError>
where
    F: Future<Output = Result<(), E>>,
{
    tokio::select! {
        biased;
        outcome = shutdown => match outcome {
            Ok(()) => Ok(true),
            Err(_) => Err(McpStdioError::new(McpStdioErrorKind::Shutdown)),
        },
        () = tokio::task::yield_now() => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::future::{pending, ready};
    use std::io;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    use super::{McpStdioErrorKind, McpStdioOutcome, McpStdioTransport};

    struct FailingReader;

    impl AsyncRead for FailingReader {
        fn poll_read(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            _buffer: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("controlled read failure")))
        }
    }

    #[derive(Default)]
    struct FailingWriter;

    impl AsyncWrite for FailingWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            _buffer: &[u8],
        ) -> Poll<io::Result<usize>> {
            Poll::Ready(Err(io::Error::other("controlled write failure")))
        }

        fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[derive(Default)]
    struct FailingFlushWriter(Vec<u8>);

    impl AsyncWrite for FailingFlushWriter {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            buffer: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.0.extend_from_slice(buffer);
            Poll::Ready(Ok(buffer.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::other("controlled flush failure")))
        }

        fn poll_shutdown(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn controlled_terminal_failures_are_closed_and_redacted() {
        let transport = McpStdioTransport::default();
        let mut reader = FailingReader;
        let mut output = Vec::new();
        let failure = transport
            .run(
                &mut reader,
                &mut output,
                pending::<Result<(), Infallible>>(),
            )
            .await
            .expect_err("reader failure must terminate");
        assert_eq!(failure.kind(), McpStdioErrorKind::Read);
        assert!(!failure.to_string().contains("controlled"));

        let mut input = &b"{}\n"[..];
        let mut writer = FailingWriter;
        let failure = transport
            .run(&mut input, &mut writer, pending::<Result<(), Infallible>>())
            .await
            .expect_err("writer failure must terminate");
        assert_eq!(failure.kind(), McpStdioErrorKind::Write);

        let mut input = &b"{}\n"[..];
        let mut writer = FailingFlushWriter::default();
        let failure = transport
            .run(&mut input, &mut writer, pending::<Result<(), Infallible>>())
            .await
            .expect_err("flush failure must terminate");
        assert_eq!(failure.kind(), McpStdioErrorKind::Flush);

        let mut input = &b""[..];
        let mut output = Vec::new();
        let outcome = transport
            .run(&mut input, &mut output, ready(Ok::<(), io::Error>(())))
            .await
            .expect("cancellation must be successful");
        assert_eq!(outcome, McpStdioOutcome::Cancelled);

        let mut input = &b""[..];
        let failure = transport
            .run(
                &mut input,
                &mut output,
                ready(Err::<(), _>(io::Error::other(
                    "controlled shutdown failure",
                ))),
            )
            .await
            .expect_err("shutdown failure must terminate");
        assert_eq!(failure.kind(), McpStdioErrorKind::Shutdown);
    }

    #[tokio::test]
    async fn controlled_encode_failure_is_closed_and_writes_nothing() {
        let transport = McpStdioTransport::default();
        let mut output = Vec::new();
        let failure = transport
            .process_frame_with_encoder(b"{}", &mut output, |_| Err(oneagent_protocol::EncodeError))
            .await
            .expect_err("encode failure must terminate");

        assert_eq!(failure.kind(), McpStdioErrorKind::Encode);
        assert!(output.is_empty());
    }
}
