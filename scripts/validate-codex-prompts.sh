#!/usr/bin/env bash

set -euo pipefail

repository_root=$(git rev-parse --show-toplevel)
cd "$repository_root"

error_count=0
validated_count=0

report_error() {
    local prompt_file=$1
    local message=$2
    printf 'ERROR %s: %s\n' "$prompt_file" "$message" >&2
    error_count=$((error_count + 1))
}

require_line() {
    local prompt_file=$1
    local expected=$2
    if ! grep -Fqx -- "$expected" "$prompt_file"; then
        report_error "$prompt_file" "missing exact line: $expected"
    fi
}

front_matter_value() {
    local prompt_file=$1
    local key=$2
    awk -v key="$key" '
        NR == 1 && $0 == "---" { active = 1; next }
        active && $0 == "---" { exit }
        active && index($0, key ":") == 1 {
            sub("^" key ":[[:space:]]*", "")
            print
            exit
        }
    ' "$prompt_file"
}

validate_front_matter_value() {
    local prompt_file=$1
    local key=$2
    local expected=$3
    local actual
    actual=$(front_matter_value "$prompt_file" "$key")
    if [[ "$actual" != "$expected" ]]; then
        report_error "$prompt_file" "$key must be $expected, found ${actual:-<missing>}"
    fi
}

validate_selected_file() {
    local prompt_file=$1
    local key=$2
    local selected
    selected=$(front_matter_value "$prompt_file" "$key")
    if [[ -z "$selected" ]]; then
        report_error "$prompt_file" "$key is missing"
    elif [[ "$selected" != docs/codex/* ]]; then
        report_error "$prompt_file" "$key must stay under docs/codex/"
    elif [[ ! -f "$selected" ]]; then
        report_error "$prompt_file" "$key does not exist: $selected"
    fi
}

validate_large_authority_selectors() {
    local prompt_file=$1
    local must_read
    must_read=$(awk '
        /^### Must read$/ { active = 1; next }
        /^### / { active = 0 }
        active { print }
    ' "$prompt_file")

    while IFS= read -r authority_line; do
        [[ -z "$authority_line" ]] && continue
        if [[ "$authority_line" == *'docs/Roadmap.md'* \
            || "$authority_line" == *'docs/Architecture.md'* \
            || "$authority_line" == *'docs/architecture/semantic-model-2.md'* ]]; then
            if [[ ! "$authority_line" =~ (sections|symbols|range|query|diff): ]]; then
                report_error "$prompt_file" \
                    "large Must read authority needs sections, symbols, range, query, or diff selector: $authority_line"
            fi
        fi
    done <<< "$must_read"
}

require_nonempty_block() {
    local prompt_file=$1
    local heading=$2
    local block
    block=$(awk -v heading="$heading" '
        $0 == heading { active = 1; next }
        active && /^#{2,3} / { exit }
        active { print }
    ' "$prompt_file")
    if ! grep -Eq '^- ' <<< "$block"; then
        report_error "$prompt_file" "$heading must contain at least one explicit item"
    fi
}

validate_prompt() {
    local prompt_file=$1

    if [[ ! -f "$prompt_file" ]]; then
        report_error "$prompt_file" "file does not exist"
        return
    fi

    if [[ $(head -n 1 "$prompt_file") != '---' ]]; then
        report_error "$prompt_file" "Prompt Contract v2 front matter must start on line 1"
    fi
    if [[ -z $(awk '/^---$/ { count += 1; if (count == 2) { print NR; exit } }' "$prompt_file") ]]; then
        report_error "$prompt_file" "Prompt Contract v2 front matter is not closed"
    fi

    validate_front_matter_value "$prompt_file" prompt_contract v2
    validate_front_matter_value "$prompt_file" fresh_context required
    validate_front_matter_value "$prompt_file" context_static_max_percent 15
    validate_front_matter_value "$prompt_file" context_authorities_max_percent 20
    validate_front_matter_value "$prompt_file" context_prework_hard_stop_percent 50
    validate_front_matter_value "$prompt_file" context_working_min_percent 35
    validate_front_matter_value "$prompt_file" context_reserve_min_percent 15

    local task_kind
    task_kind=$(front_matter_value "$prompt_file" task_kind)
    case "$task_kind" in
        investigation|architecture|implementation|review) ;;
        *) report_error "$prompt_file" "unsupported task_kind: ${task_kind:-<missing>}" ;;
    esac

    validate_selected_file "$prompt_file" profile
    validate_selected_file "$prompt_file" template

    require_line "$prompt_file" '## Reporting'
    require_line "$prompt_file" '## Context manifest'
    require_line "$prompt_file" '### Must read'
    require_line "$prompt_file" '### Lookup on demand'
    require_line "$prompt_file" '### Excluded from initial context'
    require_line "$prompt_file" '### Preflight'
    require_line "$prompt_file" '## Prerequisites / required gate'
    require_line "$prompt_file" '## Task'
    require_line "$prompt_file" '## Scope'
    require_line "$prompt_file" '### Included'
    require_line "$prompt_file" '### Excluded'
    require_line "$prompt_file" '## Acceptance criteria'
    require_line "$prompt_file" '## Task-specific validation'
    require_line "$prompt_file" '## Suggested commit message'

    require_nonempty_block "$prompt_file" '### Must read'
    require_nonempty_block "$prompt_file" '### Lookup on demand'
    require_nonempty_block "$prompt_file" '### Excluded from initial context'
    require_nonempty_block "$prompt_file" '### Preflight'

    if grep -Eq '^## (Repository [Ss]afety|Validation policy|Timing and token accounting)$' "$prompt_file"; then
        report_error "$prompt_file" \
            "permanent safety, validation, and accounting rules must remain in Core or Workflow modules"
    fi

    validate_large_authority_selectors "$prompt_file"
    validated_count=$((validated_count + 1))
}

prompt_files=()
if (( $# > 0 )); then
    prompt_files=("$@")
else
    while IFS= read -r example_file; do
        if grep -Fqx 'prompt_contract: v2' "$example_file"; then
            prompt_files+=("$example_file")
        fi
    done < <(find docs/codex/examples -type f -name '*.md' -print | sort)

    while IFS= read -r prompt_file; do
        case "$prompt_file" in
            docs/codex/prompts/sprint-6-attributes-tabular-sections/* \
                | docs/codex/prompts/sprint-7-forms-commands/* \
                | docs/codex/prompts/sprint-8-registers-queries/* \
                | docs/codex/prompts/sprint-39-change-impact-analysis/*)
                continue
                ;;
        esac

        if grep -Fqx 'prompt_contract: v2' "$prompt_file"; then
            prompt_files+=("$prompt_file")
        else
            report_error "$prompt_file" \
                "new child prompt is not Prompt Contract v2"
        fi
    done < <(
        find docs/codex/prompts -mindepth 2 -maxdepth 2 -type f \
            -name '[0-9][0-9]-*.md' ! -name '00-*' -print | sort
    )
fi

if (( ${#prompt_files[@]} == 0 )); then
    printf 'ERROR no Prompt Contract v2 files found\n' >&2
    exit 1
fi

for prompt_file in "${prompt_files[@]}"; do
    validate_prompt "$prompt_file"
done

if (( error_count > 0 )); then
    printf 'Prompt Contract v2 validation failed: %d error(s) in %d file(s).\n' \
        "$error_count" "$validated_count" >&2
    exit 1
fi

printf 'Prompt Contract v2 validation passed: %d file(s).\n' "$validated_count"
