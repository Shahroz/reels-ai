Create an API in Rust (actix) that would do the following:

- it accepts a user research instruction
- it starts a conversation with LLM (gemini flash 2.5, some openai model too)
- during this conversation it presents the problem (user instruction) and tools that are available
- the tools that are available are:
  - saving to final context (if something is valuable it should be stored in the final context)
  - selecting tools (see api_clients) to analyze the results and peform next actions
  - the main actions it should do is searching and browsing
  - the agent selects multiple tools
  - we execute the tools and provide answers 
  - and the loop continues
  - in the background there is a process which evaluates the stored context
  - the process which evaluates the stored context can also provide valuable input what's missing
  - when the background process which analyzes the context says that the the provided context is enough to solve the problem it sends termination criterium signal
  - there should be also a time limit let's say 5 minutes (configurable to finish the research)
  - overall this process accumulates the conversation - if the accumulated conversation exceeds 100k tokens it should be compacted
  - compaction is preserving the last N conversation exchanges and it summarizes the rest (we can actually remove the outputs of tools for anything beyond 10 last conversations)
