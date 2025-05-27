# API Compatibility Testing

This document describes the comprehensive API compatibility testing suite designed to ensure our Rust implementation maintains compatibility with the old Django backend API.

## Overview

The API compatibility tests serve several critical purposes:

1. **Prevent Regressions**: Catch when existing API fields are accidentally removed
2. **Document Missing Fields**: Track which fields from the old API still need implementation
3. **Ensure Field Format Consistency**: Verify JSON field naming (kebab-case) matches Django API
4. **Validate Serialization/Deserialization**: Test that models work with real API response formats

## Test Structure

### Unit Tests (`src/api_compatibility_test.rs`)

- **Individual Model Tests**: Test each model (Activity, Report, Customer, etc.) against mock Django API responses
- **Missing Fields Documentation**: Comprehensive list of 38+ missing fields that need implementation
- **Field Format Tests**: Verify kebab-case naming convention
- **Resource Name Tests**: Ensure endpoint names match Django API

### Integration Tests (`tests/api_compatibility_tests.rs`)

- **End-to-End Compatibility**: Test complete JSON:API responses with real data
- **Collection Response Tests**: Verify array responses work correctly
- **Field Naming Consistency**: Test serialization produces correct JSON format
- **Resource Name Mapping**: Validate all model endpoints

## Running the Tests

### Run All Compatibility Tests
```bash
cargo test api_compatibility
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test api_compatibility_tests --lib

# Integration tests only
cargo test --test api_compatibility_tests

# View missing fields report
cargo test test_comprehensive_missing_fields_documentation --lib -- --nocapture
```

## Current Status

### ✅ Compatible Fields
Models currently compatible with their basic Django API fields:
- **Activity**: comment, date, from-time, to-time, review, not-billable
- **Attendance**: date, from-time, to-time
- **Report**: comment, date, duration, review, not-billable, verified, billed, rejected
- **Customer**: name, archived
- **Project**: name, archived
- **Task**: name, archived
- **User**: username, email, first-name, last-name
- **Absence**: date, comment

### ❌ Missing Fields (38 total)

#### Activity Model
- `transferred` (boolean)

#### Report Model
- `added` (datetime)
- `updated` (datetime)
- `remaining-effort` (duration)

#### Customer Model
- `reference` (string)
- `email` (string)
- `website` (string)
- `comment` (string)
- `assignees` (relationship)

#### Project Model
- `reference` (string)
- `comment` (string)
- `billed` (boolean)
- `estimated-time` (duration)
- `customer-visible` (boolean)
- `amount-offered` (money)
- `amount-invoiced` (money)
- `remaining-effort-tracking` (boolean)
- `total-remaining-effort` (duration)
- `billing-type` (relationship)
- `cost-center` (relationship)
- `assignees` (relationship)

#### Task Model
- `reference` (string)
- `estimated-time` (duration)
- `amount-offered` (money)
- `amount-invoiced` (money)
- `most-recent-remaining-effort` (duration)
- `cost-center` (relationship)
- `assignees` (relationship)

#### User Model
- `tour-done` (boolean)
- `is-accountant` (boolean)
- `is-reviewer` (boolean)
- `is-active` (boolean)
- `is-staff` (boolean)
- `is-superuser` (boolean)
- `date-joined` (datetime)
- `last-login` (datetime)
- `supervisors` (relationship)
- `supervisees` (relationship)

## Test Failures and What They Mean

### Expected Failures

Some tests are designed to fail until missing fields are implemented:

```bash
test api_compatibility_test::test_project_api_compatibility ... FAILED
# Missing 'billing-type' relationship

test api_compatibility_test::test_task_api_compatibility ... FAILED
# Missing 'cost-center' relationship
```

These failures are **expected** and help track implementation progress.

### Unexpected Failures

If tests for basic compatibility fail, this indicates a regression:

```bash
test api_compatibility_test::test_activity_api_compatibility ... FAILED
# This would be a REGRESSION - basic Activity fields should work
```

## Implementation Guide

### Adding Missing Fields

1. **Update Model Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAttributes {
    pub name: String,
    pub archived: bool,
    // Add missing fields:
    pub reference: Option<String>,
    pub comment: Option<String>,
    pub billed: bool,
    #[serde(rename = "estimated-time")]
    pub estimated_time: Option<String>,
    // ... etc
}
```

2. **Update Model Relationships**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRelationships {
    pub customer: Option<RelationshipData>,
    // Add missing relationships:
    #[serde(rename = "billing-type")]
    pub billing_type: Option<RelationshipData>,
    #[serde(rename = "cost-center")]
    pub cost_center: Option<RelationshipData>,
    pub assignees: Option<RelationshipData>,
}
```

3. **Update Tests**: Remove implemented fields from the missing fields documentation test

### Field Naming Convention

- Use kebab-case in JSON: `"from-time"`, `"not-billable"`, `"estimated-time"`
- Use snake_case in Rust: `from_time`, `not_billable`, `estimated_time`
- Add `#[serde(rename = "kebab-case")]` for multi-word fields

## CI Integration

Add to your CI pipeline:

```yaml
# Test for API compatibility regressions
- name: Run API Compatibility Tests
  run: cargo test api_compatibility --all

# Generate missing fields report
- name: Check Missing API Fields
  run: cargo test test_comprehensive_missing_fields_documentation --lib -- --nocapture
```

## Benefits

1. **Early Detection**: Catch API breaking changes before they reach production
2. **Documentation**: Live documentation of what's implemented vs missing
3. **Confidence**: Deploy knowing you haven't broken API compatibility
4. **Progress Tracking**: See exactly how many fields remain to implement

## Next Steps

1. Prioritize implementing missing fields based on handler usage
2. Add new tests when adding new models
3. Update tests as fields are implemented
4. Use test failures to guide development priorities

The test suite will evolve as more fields are implemented, eventually reaching full API compatibility with the Django backend.
