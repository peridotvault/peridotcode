# Known Gaps After End-to-End Implementation

This document lists the main known gaps and limitations after implementing the end-to-end happy-path flow.

## Resolved Gaps

- **API Key Error Handling**: Added `validate_credentials` pre-flight check in the orchestrator that intercepts missing keys and shows users a clean error in the TUI advising them to check their `.env`.

## Critical Gaps (Should Address Soon)

### 1. No Real Error Recovery

**Current State**: Errors are caught and displayed but not retried

**Gap**: If the AI provider returns a transient error (rate limit, timeout), the flow fails completely without retry.

**Impact**: Users may need to manually retry prompts

**Potential Solution**: Add basic retry with exponential backoff for transient errors

```rust
// Pseudocode for retry
try_with_retry(|| provider.infer(request), max_retries = 3)
```

### 2. No Streaming Support

**Current State**: All responses are returned as complete blocks

**Gap**: Users see no progress indicator while waiting for AI response

**Impact**: Poor UX for long-running requests (10-30 seconds)

**Potential Solution**: Implement streaming responses in providers and TUI

### 3. Limited Intent Classification

**Current State**: Only 2 supported intents (CreateNewGame, AddFeature)

**Gap**: Many user requests will be classified as "Unsupported"

**Impact**: Users cannot:
- Modify existing code
- Get explanations
- Debug issues
- Ask questions

**Potential Solution**: Add more intents or use AI for general conversation

### 4. No Conversation Memory

**Current State**: Each prompt is processed independently

**Gap**: Users cannot refer to previous prompts or context

**Impact**: Poor experience for multi-step workflows

**Potential Solution**: Maintain conversation history in App state

### 5. No Validation of AI Output

**Current State**: AI response is used directly without validation

**Gap**: AI could return:
- Invalid intent classifications
- Malformed responses
- Unexpected content

**Impact**: Potential crashes or unexpected behavior

**Potential Solution**: Add validation layer for AI outputs

## Important Gaps (Should Address Eventually)

### 6. No Caching

**Current State**: Every request hits the provider API

**Gap**: Repeated similar prompts cost money and time

**Impact**: Unnecessary API costs for common operations

**Potential Solution**: Cache responses for identical prompts

### 7. No Usage Tracking

**Current State**: Token usage is returned but not stored

**Gap**: Users cannot track costs over time

**Impact**: Unexpected bills, no visibility into usage

**Potential Solution**: Log usage to a local database

### 8. Limited Model Discovery

**Current State**: Static model lists for OpenAI/Anthropic

**Gap**: New models aren't automatically available

**Impact**: Users must wait for code updates to use new models

**Potential Solution**: Fetch model lists from provider APIs

### 9. No Request Cancellation

**Current State**: Once a request starts, it cannot be cancelled

**Gap**: Users must wait for slow requests to complete

**Impact**: Poor UX for accidental prompts

**Potential Solution**: Add timeout and cancellation tokens

### 10. No Request Batching

**Current State**: One request = one API call

**Gap**: Multiple related operations each hit the API

**Impact**: Higher costs and slower execution

**Potential Solution**: Batch related operations

## Nice-to-Have Gaps

### 11. No Prompt Templates

**Current State**: System prompts are hardcoded

**Gap**: Users cannot customize AI behavior

**Impact**: One-size-fits-all AI responses

**Potential Solution**: Allow custom system prompts per project

### 12. No A/B Testing Framework

**Current State**: No way to compare model performance

**Gap**: Cannot determine which model works best for specific tasks

**Impact**: Suboptimal model selection

**Potential Solution**: Built-in benchmarking

### 13. No Offline Mode

**Current State**: Requires internet connection

**Gap**: Cannot work offline or with local models

**Impact**: No offline development capability

**Potential Solution**: Support Ollama/local models

### 14. No Multi-Provider Fallback

**Current State**: Single provider per request

**Gap**: If provider fails, no automatic fallback

**Impact**: Single point of failure

**Potential Solution**: Try alternative providers on failure

### 15. Limited Observability

**Current State**: Basic tracing only

**Gap**: No metrics, dashboards, or detailed logs

**Impact**: Hard to debug issues or optimize performance

**Potential Solution**: Add structured logging and metrics

## Architectural Gaps

### 16. No Plugin System

**Current State**: Providers are hardcoded

**Gap**: Users cannot add custom providers

**Impact**: Limited extensibility

**Potential Solution**: Dynamic provider loading

### 17. No Request Queue

**Current State**: Synchronous request processing

**Gap**: Cannot queue multiple requests

**Impact**: No batch processing capability

**Potential Solution**: Background job queue

### 18. No Request Deduplication

**Current State**: Identical requests are processed independently

**Gap**: Wasted API calls for same prompt

**Impact**: Unnecessary costs

**Potential Solution**: In-flight request tracking

### 19. No Response Post-Processing

**Current State**: Raw AI output is used

**Gap**: No extraction of structured data

**Impact**: Manual parsing required

**Potential Solution**: Response schema validation and extraction

### 20. No Context Window Management

**Current State**: No tracking of context usage

**Gap**: May exceed model context limits

**Impact**: Errors on large prompts

**Potential Solution**: Context window tracking and truncation

## Testing Gaps

### 21. Limited Integration Tests

**Current State**: Basic tests only

**Gap**: No tests for:
- Provider adapters with real APIs
- Error scenarios
- Concurrent requests
- Network failures

**Impact**: Uncovered edge cases

**Potential Solution**: Mock providers and chaos testing

### 22. No Load Testing

**Current State**: No performance benchmarks

**Gap**: Unknown throughput limits

**Impact**: May fail under load

**Potential Solution**: Load test suite

### 23. No Fuzz Testing

**Current State**: Only happy path tested

**Gap**: Unexpected inputs may crash system

**Impact**: Security/reliability issues

**Potential Solution**: Fuzz testing for input validation

## Summary

**Total Gaps**: 23

**By Priority**:
- Critical: 5
- Important: 5
- Nice-to-have: 5
- Architectural: 4
- Testing: 4

**Recommended Next Steps**:
1. Add basic retry logic for transient errors
2. Implement streaming for better UX
3. Expand intent classification
4. Add conversation memory
5. Add output validation

These gaps represent opportunities for improvement while maintaining the "happy path first" philosophy. Address them incrementally based on user feedback and observed pain points.
