/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { AttachmentType } from './AttachmentType';
/**
 * Represents a file or data attached to a research request.
 */
export type Attachment = {
    /**
     * The kind of the attachment, including its specific data.
     */
    kind: AttachmentType;
    /**
     * An optional title for the attachment (e.g., "Document about X").
     */
    title?: string | null;
};

