# Theoretical Context: RAG Context Faithfulness and Hallucination Prevention

## Production Incident: Air Canada Chatbot False Refund Advice (2022)

In November 2022, a passenger booked a last-minute flight with Air Canada to attend a family bereavement after consulting the airline's automated AI customer service chatbot. The chatbot explicitly instructed the passenger that they could purchase a standard fare ticket immediately and submit an application for retroactive bereavement discount refund within 90 days. When the customer filed the refund claim, Air Canada refused payment, pointing out that their official written bereavement policy strictly prohibited retroactive discount applications. In the landmark small claims tribunal ruling (*Moffatt v. Air Canada*, 2024), the court rejected Air Canada's argument that the chatbot was a separate legal entity responsible for its own actions, holding the airline liable for negligent misrepresentation. The incident demonstrated the severe financial, legal, and reputational hazards of deploying AI systems without automated context grounding and faithfulness verification.

## The Underlying Mechanism

Retrieval-Augmented Generation (RAG) architecture combines semantic search over an indexed document corpus (vector database) with Large Language Model (LLM) parametric knowledge generation:

1. **The Hallucination Failure Mode**: When an LLM generates answers to user queries, it relies on probabilistic token prediction. If the system prompt or retrieval pipeline fails to strictly constrain the model to the retrieved context chunks, the LLM will hallucinate convincing, plausible-sounding assertions that directly contradict the grounding knowledge base.
2. **Deterministic Context Faithfulness Testing**: In modern GenAI quality engineering, automated tests must verify that every factual claim in the model's output can be directly attributed to the retrieved context documents:
   - **Context Recall**: Ensuring the retrieval step fetched all relevant policy clauses.
   - **Faithfulness / Groundedness**: Asserting that generated answers do not extrapolate beyond or fabricate terms missing from the reference documents.
   - **Negative Assertion Boundaries**: Verifying that when source context lacks sufficient information, the model replies with an explicit disclaimer rather than fabricating an answer.

```
[Anti-Pattern: Ungrounded Probabilistic Generation]
User Query ──► Vector Search ──► Retrieved Context (No Refund Rule)
                                         │
                                         ▼
                                   [LLM Generator]
                            (Hallucinates "90-Day Refund")
                                         │
                                         ▼
                             Court Liability & Loss! ❌

[Resilient SDET Pattern: Automated RAG Faithfulness Oracle]
User Query ──► Vector Search ──► Retrieved Context (No Refund Rule)
                                         │
                                         ▼
                                   [LLM Generator]
                                         │
                                         ▼
                     [Automated Faithfulness Assertion]
                     Is Claim ∈ Grounding Context? ──► FAIL / REJECT ✅
```

Rigorous automated assertions on context adherence ensure that enterprise AI applications remain factual, compliant, and legally grounded.

You will now simulate this in the Crucible: assert RAG context faithfulness and detect ungrounded model hallucinations using automated Playwright test oracles.
