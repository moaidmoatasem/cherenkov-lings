# Theoretical Context: LLM Hallucination Evaluation & Grounding

## Real-World Incident Case Study
In *Moffatt v. Air Canada (2024)*, the airline argued it was not responsible for its chatbot hallucinating retroactive bereavement discounts. The Canadian Civil Resolution Tribunal ruled that a company is legally responsible for all representations made by its automated agents.

## Protocol & Runtime Mechanism
Generative LLMs sample token probability distributions:

$$P(w_t \mid w_1, \dots, w_{t-1})$$

When context retrieval fails or prompt entropy is high, models invent plausible-sounding falsehoods (hallucinations). Automated QA must assert on:
1. Citation ground truth presence.
2. Negative keyword guardrails.
3. RAG triangulation scoring.

```
  [ User Query ] --? [ RAG Retriever ] --? [ Prompt Context ] --? [ LLM Generator ]
                            ¦                                            ¦
                            ?                                            ?
                    Source Documents                            Hallucination Check
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=genai-qa` and verify the grounding assertion against the Micro-Crucible RAG endpoint.
