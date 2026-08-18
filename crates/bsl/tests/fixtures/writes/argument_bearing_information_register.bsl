Procedure CreateHistoryOfCustomsClearanceStatusesRecords(Cancel)

	If Cancel Then
		Return;
	EndIf;

	If TypeOf(DocumentBasis) = Type("DocumentRef.CustomImportDeclaration") Then

		NewRecord = RegisterRecords.HistoryOfCustomsClearanceStatuses.Add();
		NewRecord.CustomsDeclaration = DocumentBasis;
		NewRecord.Period = PointInTime().Date;
		NewRecord.Recorder = Ref;
		NewRecord.Status = Enums.CustomsClearanceStatuses.Released;
		RegisterRecords.HistoryOfCustomsClearanceStatuses.Write(True);

	EndIf;

EndProcedure
