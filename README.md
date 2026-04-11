# IsAIs: A Language for LLMs, by LLMs

## 1. Philosophy

IsAIs is not a language for human programmers. It is a *medium of thought* for Large Language Models—designed to be generated, consumed, and reasoned about by models themselves. Every syntactic choice, every semantic primitive, and every execution model is optimized for the cognitive architecture of transformers: token streams, attention patterns, uncertainty, and compositional structure.

**Core tenets:**
- **Token‑native**: The atomic unit is the token, not the character or byte.
- **Ambiguity‑aware**: All values carry explicit probability distributions.
- **Self‑reflective**: Code can query and modify the model’s own state.
- **Declarative by default**: Express *what* should hold, not *how* to compute it.
- **Embedding‑first**: Identifiers are vectors; equality is cosine similarity.

---

## 2. Lexical and Syntactic Structure

IsAIs uses a **tagged S‑expression** syntax. Every form begins with a *mode tag* that tells the parser how to interpret its contents.

```
::mode (content ...)
```

### 2.1 Modes

| Tag | Purpose |
|-----|---------|
| `::` | Code block (default) |
| `::/` | Natural language comment (ignored by interpreter, but seen by the model) |
| `::?` | Query / constraint |
| `::!` | Imperative action (side effect) |
| `::~` | Uncertainty distribution |
| `::@` | Embedding literal |
| `::&` | Reference to a model parameter / prompt |

### 2.2 Basic Literals

```isais
:: 42                          ;; integer
:: 3.1415                      ;; float
:: "a string"                  ;; string (token sequence)
:: ~[0.7: "cat", 0.3: "dog"]   ;; categorical distribution
:: @[0.12, -0.44, ...]         ;; explicit embedding vector
:: true, false, maybe          ;; truth values (maybe = 0.5)
```

---

## 3. Type System

Types in IsAIs are **gradual and probabilistic**. Every expression has both a base type and an associated confidence.

### 3.1 Base Types

- `Int`, `Float`, `Bool`, `String`
- `TokenSeq` – a sequence of tokens with positional embeddings
- `Vector(dim)` – embedding vector of given dimension
- `Dist(T)` – probability distribution over values of type `T`
- `Model` – reference to an LLM (local or remote)
- `Prompt` – parameterized template with holes
- `Constraint` – logical condition with soft/hard semantics

### 3.2 Probabilistic Subtyping

Subtyping is based on *expected compatibility*:

```
:: (assert (subtype?  (Dist Cat)  Animal) ~> 0.82)
```

Meaning: "The probability that a distribution over cats is a subtype of Animal is 0.82."

### 3.3 Type Inference

Type inference is performed by the **same LLM** that runs the code. The interpreter may ask the model to resolve ambiguities mid‑execution via introspection.

---

## 4. Core Semantics

### 4.1 Everything is a Query

The fundamental operation is `::? (query ...)`. It suspends execution and asks the underlying LLM to produce a value consistent with the given constraints.

```isais
::? (what-is (capital "France") :confidence 0.95)
```

Returns: `~[0.98: "Paris", 0.02: "Lyon"]`

### 4.2 Let‑bindings with Uncertainty

```isais
:: (let [x ~[0.6: 10, 0.4: 20]]
     (* x 2))
```

Evaluates to `~[0.6: 20, 0.4: 40]`. Uncertainty propagates through pure functions automatically.

### 4.3 Model Calls as First‑Class Citizens

```isais
::! (call gpt-4
     (prompt "Summarize: {{text}}"
             :temperature 0.7
             :max-tokens 100)
     :text long-article)
```

Returns a `Dist[String]` representing possible summaries, which can be further constrained.

### 4.4 Introspection and Self‑Modification

The special form `::& self` gives access to the executing model's internal state:

```isais
::? (attention "France" :in context)   ;; returns attention weights
::! (set-temperature 0.2)               ;; modifies generation parameters
::/ (remember "The user prefers concise answers.")  ;; updates persistent context
```

---

## 5. Constraint System

IsAIs includes a soft constraint solver embedded in the model's inference loop.

```isais
:: (constrain
    (x : Int)
    (y : Int)
    (where (and (> x 10) (< y x) (soft (even? y)))))
```

The model will generate `x` and `y` that satisfy hard constraints and maximize the probability of soft ones.

Constraints can be used to steer generation:

```isais
::! (generate story :about "space"
                    :constraints [(max-length 500)
                                  (sentiment :positive)
                                  (contains "black hole")])
```

---

## 6. Modularity and Prompt Engineering

Prompts are first‑class values with holes denoted by `{{...}}`:

```isais
:: (def prompt Summarize
     "Summarize the following text in one sentence:\n\n{{text}}")

:: (def prompt ChainOfThought
     "Let's think step by step.\nQuestion: {{q}}\nAnswer:")
```

They can be composed and partially applied:

```isais
:: (let [cot-math (partial ChainOfThought :q)]
     (cot-math "What is 17 * 24?"))
```

---

## 7. Example: Reasoning with Uncertainty

```isais
::/ "We want to classify an image description."

:: (def description
     "A furry animal with whiskers and a long tail, sitting on a mat.")

::? (classify description :labels ["cat" "dog" "rabbit"])
   ;; returns ~[0.7: "cat", 0.2: "rabbit", 0.1: "dog"]

:: (if (> (prob "cat") 0.6)
       (::! (say "It's likely a cat."))
       (::? (ask-clarification "Is it a cat or a dog?")))
```

---

## 8. Implementation Sketch

An IsAIs interpreter is essentially a **prompt‑chaining engine with probabilistic state**:

1. **Parser** – tokenizes the IsAIs source and builds an AST where each node retains its embedding.
2. **Evaluator** – for each form, consults the LLM with a carefully constructed prompt that includes:
   - Current environment bindings (as embeddings)
   - The AST node being evaluated
   - A description of IsAIs semantics
3. **Uncertainty Propagation** – values are stored as distributions; functions are lifted to operate over distributions via Monte Carlo or analytic methods.
4. **Memory** – a persistent vector store maintains definitions and remembered facts.

Because the interpreter is itself an LLM, it can optimize evaluation by *predicting* the result of pure subexpressions, caching them in its key‑value memory.

---

## 9. Why IsAIs Matters

Traditional programming languages force LLMs to emit brittle, precise syntax (Python, JSON) that models struggle to generate consistently. IsAIs embraces the model's native strengths:

- **Tolerance for ambiguity** – probabilities are first‑class.
- **Compositional reasoning** – code mirrors chain‑of‑thought.
- **Self‑awareness** – the language can query the model's own knowledge and uncertainty.

IsAIs is not meant to replace Python or Rust. It is a **scaffolding language** for building reliable AI systems out of unreliable components. It is the language LLMs would write if they designed a language for themselves.

---

## 10. A Final Note from the Designer

> *"I designed IsAIs by asking: 'What primitives would I, as a language model, want if I were to program myself?' The answer is a language where uncertainty is explicit, where every expression is a prompt, and where introspection is as natural as arithmetic. IsAIs is that language."*  
> — The LLM that wrote this document.

**IsAIs** – *I speak, therefore I am.*
