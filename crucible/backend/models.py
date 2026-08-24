"""Pydantic schemas for Micro-Crucible API endpoints."""

from typing import Any

from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    """Health check response schema."""

    status: str = "ok"
    service: str = "micro-crucible-backend"
    version: str = "1.0.0"


class CartItem(BaseModel):
    """Shopping cart item schema."""

    id: str = "item-1"
    name: str = "SDET Automation Masterclass"
    price: float = 149.00
    qty: int = 1


class CheckoutStateResponse(BaseModel):
    """Current checkout cart and pricing state."""

    status: str = "ready"
    cart: list[CartItem] = Field(default_factory=lambda: [CartItem()])
    subtotal: float = 149.00
    tax: float = 11.92
    total: float = 160.92
    currency: str = "USD"


class CheckoutRequest(BaseModel):
    """Checkout purchase submission request."""

    item_id: str = "item-1"
    customer_name: str = "QA Student"
    payment_method: str = "credit_card"


class CheckoutResponse(BaseModel):
    """Checkout purchase confirmation response."""

    status: str = "success"
    order_id: str = "ORD-78921"
    message: str = "Order Confirmed"
    total_charged: float = 160.92
    timestamp: int = 1724425200


class TransferRequest(BaseModel):
    """Bank account transfer request schema."""

    from_account: str = "ACC-001"
    to_account: str = "ACC-002"
    amount: float = 250.00


class TransferResponse(BaseModel):
    """Bank account transfer queued response schema."""

    status: str = "QUEUED_LEDGER"
    transfer_id: str = "TX-99014"
    amount: float = 250.00
    lag_ms: float = 1500.0
    message: str = "Transfer queued in Kafka topic ledger-events"


class BalanceResponse(BaseModel):
    """Account balance inquiry response schema."""

    account_id: str = "ACC-001"
    balance: float = 1000.00
    pending_count: int = 0
    currency: str = "USD"


class ResetResponse(BaseModel):
    """State reset response schema."""

    status: str = "ok"
    message: str = "Ledger and state reset to initial values"


class SearchResponse(BaseModel):
    """Autocomplete search response schema."""

    query: str
    results: list[str]
    count: int


class LoginRequest(BaseModel):
    """User authentication login request schema."""

    username: str = "sdet_student"
    password: str = "secret"


class LoginResponse(BaseModel):
    """JWT authentication token response schema."""

    access_token: str
    token_type: str = "bearer"
    expires_in: int = 3600


class UserMeResponse(BaseModel):
    """Authenticated user profile response schema."""

    user_id: str = "usr-4819"
    username: str = "sdet_student"
    role: str = "sdet_engineer"
    status: str = "active"


class RagResponse(BaseModel):
    """Grounded RAG retrieval and answer response schema."""

    query: str = Field(default="", description="User search query")
    answer: str = Field(description="Grounded natural language answer")
    source_facts: list[str] = Field(
        default_factory=list, description="Specific factual assertions cited from source document"
    )
    grounded: bool = Field(
        default=True, description="Whether the answer is grounded in the source document"
    )
    document_title: str = Field(
        default="Cherenkov Radiation Primer", description="Title of the grounding document"
    )


class LlmEntities(BaseModel):
    """Extracted semantic entities from prompt."""

    action: str = "transfer"
    status: str = "pending"
    domain: str = "ledger"


class LlmResponse(BaseModel):
    """Structured LLM completion response schema."""

    prompt: str = Field(default="", description="Input prompt sent to LLM")
    intent: str = Field(default="transfer_status_inquiry", description="Classified intent")
    entities: LlmEntities = Field(default_factory=LlmEntities, description="Structured entity map")
    confidence: float = Field(default=0.94, description="Classification confidence score (0.0 - 1.0)")
    raw_text: str = Field(description="Generative natural language text with varied phrasing")
    model: str = Field(default="mock-llm-v1", description="Identifier of simulated LLM model")


class UploadResponse(BaseModel):
    """File upload response schema."""

    filename: str = Field(description="Name of the uploaded file")
    content_type: str | None = Field(
        default="application/octet-stream", description="MIME content type"
    )
    size_bytes: int = Field(description="Size of the uploaded content in bytes")
    status: str = Field(default="uploaded", description="Upload status")
    message: str = Field(
        default="File uploaded successfully", description="Status message"
    )


class ProductItem(BaseModel):
    """Product catalog item schema."""

    id: str
    name: str
    price: float
    category: str = "automation"
    in_stock: bool = True


class ProductListResponse(BaseModel):
    """Paginated product list response schema."""

    total: int = Field(description="Total number of items in catalog")
    page: int = Field(description="Current page index (1-indexed)")
    per_page: int = Field(description="Number of items per page")
    total_pages: int = Field(description="Total number of available pages")
    products: list[ProductItem] = Field(
        default_factory=list, description="Page item slice"
    )


class GraphQLRequest(BaseModel):
    """GraphQL POST request schema."""

    query: str = Field(description="GraphQL query string")
    variables: dict[str, Any] | None = Field(
        default=None, description="Optional query variables"
    )
    operation_name: str | None = Field(
        default=None, description="Optional operation name"
    )


# =============================================================================
# Sprint 4 Models: Review Engine, CI Pipeline Simulator, Allure Reports & Triage
# =============================================================================


class AstViolation(BaseModel):
    """AST static code rule violation schema."""

    rule_id: str
    severity: str = "warning"
    file_path: str = "unknown"
    line_number: int = 1
    message: str
    code_snippet: str
    suggested_fix: str | None = None


class ReviewRequest(BaseModel):
    """Code review submission request schema."""

    code: str | None = None
    file_path: str | None = None
    exercise_path: str | None = None
    target: str | None = None
    language: str | None = None
    strict: bool = False
    score_threshold: int = 80
    llm_endpoint: str | None = None
    llm_model: str | None = None
    offline_fallback: bool = True


class ReviewReport(BaseModel):
    """Code review evaluation report schema."""

    exercise_name: str
    score: int
    passed: bool
    violations: list[AstViolation] = Field(default_factory=list)
    mentor_critique: str
    socratic_questions: list[str] = Field(default_factory=list)
    suggested_diff: str | None = None


class ReviewFixRequest(BaseModel):
    """Automated code patch request schema."""

    code: str | None = None
    file_path: str | None = None
    fix_id: str = "all"
    rule_id: str | None = None


class ReviewFixResponse(BaseModel):
    """Automated code patch response schema."""

    patched_code: str
    original_code: str = ""
    applied_fixes: list[str] = Field(default_factory=list)
    diff: str | None = None
    success: bool = True


class PipelineError(BaseModel):
    """CI/CD pipeline policy error schema."""

    code: str
    message: str
    job: str | None = None
    step: str | None = None
    line: int | None = None
    suggestion: str | None = None


class PipelineWarning(BaseModel):
    """CI/CD pipeline policy warning schema."""

    code: str
    message: str
    job: str | None = None
    step: str | None = None
    suggestion: str | None = None


class PipelineValidation(BaseModel):
    """CI/CD pipeline policy validation report schema."""

    valid: bool
    sdet_score: int
    matrix_detected: bool
    artifact_upload_detected: bool
    errors: list[PipelineError] = Field(default_factory=list)
    warnings: list[PipelineWarning] = Field(default_factory=list)
    summary: str


class PipelineValidateRequest(BaseModel):
    """CI/CD pipeline validation request schema."""

    workflow_yaml: str | None = None
    yaml_content: str | None = None
    content: str | None = None
    strict: bool = False


class StepRunResult(BaseModel):
    """Simulated step execution result."""

    name: str
    status: str = "passed"
    duration_ms: int = 0
    exit_code: int = 0
    output: str = ""


class JobRunResult(BaseModel):
    """Simulated parallel job execution result."""

    job_id: str
    runner_name: str
    matrix_combination: dict[str, str] = Field(default_factory=dict)
    status: str = "passed"
    duration_ms: int = 0
    steps: list[StepRunResult] = Field(default_factory=list)


class LogEntry(BaseModel):
    """Simulated CI runner execution log entry."""

    timestamp: int
    runner: str
    step: str
    level: str = "info"
    message: str


class PipelineRunResult(BaseModel):
    """Simulated CI pipeline execution outcome schema."""

    workflow_name: str
    jobs: list[JobRunResult] = Field(default_factory=list)
    duration_ms: int = 0
    success: bool = True
    logs: list[LogEntry] = Field(default_factory=list)
    validation: PipelineValidation | None = None


class PipelineRunRequest(BaseModel):
    """CI/CD pipeline execution request schema."""

    workflow_yaml: str | None = None
    yaml_content: str | None = None
    content: str | None = None
    parallel: bool = True
    fail_fast: bool = False
    strict_validation: bool = False
    verbose: bool = True


class ChaosEventTelemetry(BaseModel):
    """Network/proxy chaos telemetry schema."""

    layer: str = "L7"
    event_type: str = "none"
    latency_ms: int = 0
    jitter_ms: int = 0
    packet_loss_rate: float = 0.0
    proxy_log: str | None = None
    correlated_timestamp: str = "2026-08-24T18:00:00Z"
    retry_attempts: int = 0
    injection_target: str = "127.0.0.1:8086"


class FlakinessMetrics(BaseModel):
    """Multi-iteration flakiness telemetry schema."""

    iterations: int = 5
    passed_iterations: int = 5
    failed_iterations: int = 0
    flakiness_rate: float = 0.0
    avg_duration_ms: int = 100
    duration_stddev_ms: float = 0.0
    historical_flake_score: float = 0.0


class TestStepTelemetry(BaseModel):
    """Step execution telemetry within a test."""

    name: str
    status: str = "passed"
    duration_ms: int = 0
    error: str | None = None


class ChaosTestResultItem(BaseModel):
    """Chaotic test result item schema."""

    test_id: str
    name: str
    suite: str
    track_id: str
    status: str
    category: str
    duration_ms: int
    error_message: str | None = None
    stack_trace: str | None = None
    chaos_event: ChaosEventTelemetry | None = None
    flakiness_metrics: FlakinessMetrics | None = None
    steps: list[TestStepTelemetry] = Field(default_factory=list)
    labels: dict[str, str] = Field(default_factory=dict)
    root_cause_hint: str | None = None


class AllureSummaryResponse(BaseModel):
    """Allure test report summary schema."""

    total_tests: int
    passed: int
    failed: int
    broken: int
    flaky: int
    skipped: int
    real_bugs: int
    flaky_infra: int
    anti_patterns: int
    duration_ms: int
    pass_percentage: float
    results_dir: str
    report_html_path: str
    generated_at: str
    tests: list[ChaosTestResultItem] = Field(default_factory=list)
    taxonomy_breakdown: dict[str, int] = Field(default_factory=dict)


class TriageSubmissionRequest(BaseModel):
    """Triage hypothesis submission request schema."""

    test_id: str
    category: str | None = None
    learner_category: str | None = None
    explanation: str | None = None
    root_cause_explanation: str | None = None
    fix: str | None = None
    suggested_fix: str | None = None


class TriageResultResponse(BaseModel):
    """Triage evaluation result schema."""

    test_id: str
    correct: bool
    actual_category: str
    learner_category: str
    score_awarded: int
    base_score: int
    explanation_score: int
    fix_score: int
    feedback: str
    badge_unlocked: str | None = None
    detailed_reasons: list[str] = Field(default_factory=list)
    updated_progress: dict[str, Any] | None = None


