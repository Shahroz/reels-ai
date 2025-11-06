/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { CompactionPolicy } from './CompactionPolicy';
import type { EvaluationPolicy } from './EvaluationPolicy';
export type SessionConfig = {
    /**
     * Policy defining how conversation history should be compacted.
     */
    compaction_policy: CompactionPolicy;
    /**
     * Policy defining how the session's progress or final output is evaluated.
     */
    evaluation_policy: EvaluationPolicy;
    /**
     * The initial instruction provided by the user to start the session.
     */
    initial_instruction?: string | null;
    /**
     * Number of recent conversation exchanges to preserve during context compaction.
     */
    preserve_exchanges: number;
    /**
     * The maximum duration allowed for the session.
     */
    time_limit: string;
    /**
     * A potential threshold related to token usage (specific interpretation depends on implementation).
     */
    token_threshold: number;
};

