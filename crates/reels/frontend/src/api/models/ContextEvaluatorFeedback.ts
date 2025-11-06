/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
export type ContextEvaluatorFeedback = {
    /**
     * Flag indicating whether the context is deemed insufficient or needs updating.
     * True if updates/clarifications are needed, false otherwise.
     */
    needs_update: boolean;
    /**
     * A score indicating the relevance or sufficiency of the current context.
     * Ranges from 0.0 (insufficient) to 1.0 (sufficient).
     */
    relevance_score: number;
    /**
     * A list of suggested next steps for the agent based on the context analysis.
     */
    suggestions: Array<string>;
};

