/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { BinaryAttachment } from './BinaryAttachment';
import type { ImageAttachment } from './ImageAttachment';
import type { PdfAttachment } from './PdfAttachment';
import type { TextAttachment } from './TextAttachment';
import type { VideoUrlAttachment } from './VideoUrlAttachment';
/**
 * Enum representing the type of an attachment.
 */
export type AttachmentType = ({
    /**
     * Plain text content.
     */
    Text: TextAttachment;
} | {
    /**
     * PDF document (placeholder for future support).
     */
    Pdf: PdfAttachment;
} | {
    /**
     * Image file (placeholder for future support).
     */
    Image: ImageAttachment;
} | {
    /**
     * URL to a video (e.g., YouTube) (placeholder for future support).
     */
    VideoUrl: VideoUrlAttachment;
} | {
    /**
     * Generic binary file (placeholder for future support).
     */
    Binary: BinaryAttachment;
});

