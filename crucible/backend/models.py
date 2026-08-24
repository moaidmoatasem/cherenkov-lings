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

