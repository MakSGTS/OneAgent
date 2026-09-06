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

validate_future_sprint_execution_loop() {
    local prompt_file=$1
    local contract_block
    local contract_count
    local matrix_count
    local matrix_line
    local matrix_value
    local matrix_path
    local matrix_heading
    local matrix_heading_count
    local gate_count
    local gate_line
    local gate_value
    local gate_prompt
    local gate_prerequisite
    local gate_artifact
    local gate_commit
    local gate_extra
    local baseline_count=0
    local full_gate_total=0
    local prompt_directory

    require_line "$prompt_file" '## Sprint efficiency contract'
    contract_block=$(awk '
        $0 == "## Sprint efficiency contract" { active = 1; next }
        active && /^## / { exit }
        active { print }
    ' "$prompt_file")

    contract_count=$(awk '$0 == "sprint_efficiency_contract: v1" { count += 1 } END { print count + 0 }' \
        <<< "$contract_block")
    if [[ "$contract_count" != 1 ]]; then
        report_error "$prompt_file" \
            "Sprint efficiency contract must contain exactly one sprint_efficiency_contract: v1 record"
    fi

    matrix_count=$(awk '/^adr_invariant_matrix: / { count += 1 } END { print count + 0 }' \
        <<< "$contract_block")
    matrix_line=$(awk '/^adr_invariant_matrix: / { print; exit }' <<< "$contract_block")
    matrix_value=${matrix_line#adr_invariant_matrix: }
    matrix_path=${matrix_value%%::*}
    matrix_heading=${matrix_value#*::}
    if [[ "$matrix_count" != 1 ]] \
        || ! grep -Eq '^adr_invariant_matrix: docs/[A-Za-z0-9._/-]+::[^|]+$' \
            <<< "$contract_block"; then
        report_error "$prompt_file" \
            "Sprint efficiency contract needs one non-empty repository matrix path and section"
    elif [[ ! -f "$matrix_path" ]]; then
        report_error "$prompt_file" "ADR-invariant matrix file does not exist: $matrix_path"
    else
        matrix_heading_count=$(awk -v expected="$matrix_heading" '
            /^#{1,6} / {
                heading = $0
                sub(/^#{1,6} /, "", heading)
                if (heading == expected) {
                    count += 1
                }
            }
            END { print count + 0 }
        ' "$matrix_path")
        if [[ "$matrix_heading_count" != 1 ]]; then
            report_error "$prompt_file" \
                "ADR-invariant matrix selector must resolve to one exact Markdown heading"
        fi
    fi

    gate_count=$(awk '/^design_review_gate: / { count += 1 } END { print count + 0 }' \
        <<< "$contract_block")
    gate_line=$(awk '/^design_review_gate: / { print; exit }' <<< "$contract_block")
    if [[ "$gate_count" != 1 ]]; then
        report_error "$prompt_file" \
            "Sprint efficiency contract needs exactly one design_review_gate record"
    elif [[ "$gate_line" == 'design_review_gate: none' ]]; then
        if [[ $(awk -v expected="$gate_line" '$0 == expected { count += 1 } END { print count + 0 }' \
            docs/Roadmap.md) != 1 ]]; then
            report_error "$prompt_file" \
                "design_review_gate: none must occur exactly once in docs/Roadmap.md"
        fi
    else
        gate_value=${gate_line#design_review_gate: }
        IFS='|' read -r gate_prompt gate_prerequisite gate_artifact gate_commit gate_extra \
            <<< "$gate_value"
        if [[ -z "$gate_prompt" || -z "$gate_prerequisite" || -z "$gate_artifact" \
            || -z "$gate_commit" || -n "$gate_extra" ]]; then
            report_error "$prompt_file" \
                "design_review_gate must be none or contain exactly four non-empty fields"
        else
            if [[ ! -f "$gate_prompt" ]]; then
                report_error "$prompt_file" "design-review prompt does not exist: $gate_prompt"
            elif [[ $(front_matter_value "$gate_prompt" task_kind) != review ]]; then
                report_error "$prompt_file" "design-review prompt must use task_kind: review"
            fi
            if [[ $(dirname "$gate_prompt") != $(dirname "$prompt_file") ]]; then
                report_error "$prompt_file" "design-review prompt must belong to the same sprint suite"
            fi
            if [[ ! "$gate_artifact" =~ ^docs/reviews/[A-Za-z0-9._/-]+\.md$ ]]; then
                report_error "$prompt_file" \
                    "design-review artifact must be a repository-relative docs/reviews Markdown path"
            fi
            if [[ $(awk -v expected="$gate_line" '$0 == expected { count += 1 } END { print count + 0 }' \
                docs/Roadmap.md) != 1 ]]; then
                report_error "$prompt_file" \
                    "design_review_gate record must occur exactly once in docs/Roadmap.md"
            fi
        fi
    fi

    while IFS= read -r baseline_line; do
        local baseline_value
        local baseline_prompt
        local expected_paths
        local expected_churn
        local expected_binaries
        local focused_check_count
        local full_gate_count
        local baseline_extra

        [[ -z "$baseline_line" ]] && continue
        baseline_count=$((baseline_count + 1))
        baseline_value=${baseline_line#implementation_baseline: }
        IFS='|' read -r baseline_prompt expected_paths expected_churn \
            expected_binaries focused_check_count full_gate_count baseline_extra \
            <<< "$baseline_value"

        if [[ -z "$baseline_prompt" || -z "$expected_paths" || -z "$expected_churn" \
            || -z "$expected_binaries" || -z "$focused_check_count" \
            || -z "$full_gate_count" || -n "$baseline_extra" ]]; then
            report_error "$prompt_file" \
                "implementation_baseline must contain exactly six non-empty fields"
            continue
        fi
        if [[ ! "$expected_paths" =~ ^[0-9]+$ || ! "$expected_churn" =~ ^[0-9]+$ \
            || ! "$focused_check_count" =~ ^[0-9]+$ \
            || ! "$full_gate_count" =~ ^[01]$ ]]; then
            report_error "$prompt_file" \
                "implementation baseline counts must be non-negative integers and full gate 0 or 1"
        else
            full_gate_total=$((full_gate_total + full_gate_count))
        fi
        if [[ "$expected_binaries" != none \
            && ! "$expected_binaries" =~ ^[A-Za-z0-9._/-]+(,[A-Za-z0-9._/-]+)*$ ]]; then
            report_error "$prompt_file" \
                "expected binary paths must be none or a comma-separated repository-relative inventory"
        fi
        if [[ ! -f "$baseline_prompt" ]]; then
            report_error "$prompt_file" "implementation prompt does not exist: $baseline_prompt"
        elif [[ $(front_matter_value "$baseline_prompt" task_kind) != implementation ]]; then
            report_error "$prompt_file" \
                "implementation baseline prompt must use task_kind: implementation"
        fi
        if [[ $(dirname "$baseline_prompt") != $(dirname "$prompt_file") ]]; then
            report_error "$prompt_file" "implementation baseline must belong to the same sprint suite"
        fi
        if [[ $(awk -v expected="$baseline_line" '$0 == expected { count += 1 } END { print count + 0 }' \
            docs/Roadmap.md) != 1 ]]; then
            report_error "$prompt_file" \
                "implementation_baseline record must occur exactly once in docs/Roadmap.md"
        fi
    done < <(awk '/^implementation_baseline: / { print }' <<< "$contract_block")

    if (( baseline_count == 0 )); then
        report_error "$prompt_file" \
            "Sprint efficiency contract needs at least one implementation_baseline record"
    elif (( full_gate_total != 1 )); then
        report_error "$prompt_file" \
            "implementation_baseline full-gate counts must sum to exactly 1"
    fi

    prompt_directory=$(dirname "$prompt_file")
    while IFS= read -r child_prompt; do
        if [[ $(front_matter_value "$child_prompt" task_kind) == implementation ]]; then
            local matching_baselines
            matching_baselines=$(awk -v prefix="implementation_baseline: $child_prompt|" \
                'index($0, prefix) == 1 { count += 1 } END { print count + 0 }' \
                <<< "$contract_block")
            if [[ "$matching_baselines" != 1 ]]; then
                report_error "$prompt_file" \
                    "implementation child needs exactly one baseline: $child_prompt"
            fi
        fi
    done < <(find "$prompt_directory" -maxdepth 1 -type f \
        -name '[0-9][0-9]-*.md' ! -name '00-*' -print | sort)

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

    while IFS= read -r master_file; do
        if [[ "$master_file" =~ /sprint-([0-9]+)- ]] \
            && (( 10#${BASH_REMATCH[1]} >= 41 )); then
            prompt_files+=("$master_file")
        fi
    done < <(
        find docs/codex/prompts -mindepth 2 -maxdepth 2 -type f \
            -name '00-sprint-*-execution-loop.md' -print | sort
    )
fi

if (( ${#prompt_files[@]} == 0 )); then
    printf 'ERROR no Codex prompt files found\n' >&2
    exit 1
fi

for prompt_file in "${prompt_files[@]}"; do
    if [[ "$prompt_file" =~ /sprint-([0-9]+)-[^/]+/00-sprint-[0-9]+-execution-loop\.md$ ]] \
        && (( 10#${BASH_REMATCH[1]} >= 41 )); then
        validate_future_sprint_execution_loop "$prompt_file"
    else
        validate_prompt "$prompt_file"
    fi
done

if (( error_count > 0 )); then
    printf 'Codex prompt validation failed: %d error(s) in %d file(s).\n' \
        "$error_count" "$validated_count" >&2
    exit 1
fi

printf 'Codex prompt validation passed: %d file(s).\n' "$validated_count"
