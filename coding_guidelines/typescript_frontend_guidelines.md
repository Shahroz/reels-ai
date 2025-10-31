//! TypeScript Frontend Coding Standards
//!
//! This document defines the coding standards for the TypeScript frontend code. It aims to ensure consistency, modularity, and clarity across all components that build the user-facing parts of the application.
//!
//! Revision History:
//! - 2025-04-13T13:38:44Z @AI: Created initial guidelines based on rust_backend.md, adjusted for TypeScript and frontend specifics.
//! - Further revisions should document updates to styling, testing, or architectural best practices.

# TypeScript Frontend Coding Standards

**Important Note:** These guidelines describe *how* to write code. Do not copy the text of these guidelines into your source files. Apply the principles outlined below.


## 1. Granularity: One Logical Item Per File

- **Rule:** Each TypeScript (.ts or .tsx) file should export exactly one primary logical item — be it a component, a service, a utility function, or a constant.
- **Definition of "Logical Item":** A single React component, service class, or a related group of functions that collaborate to perform a specific task.
- **File Naming:** Files should use kebab-case or camelCase matching the exported item (e.g., `user-profile.tsx` for a component named UserProfile, or `authService.ts` for an authentication service).

## 2. File Preamble Documentation

- **Rule:** Every TypeScript file must begin with a brief file-level documentation comment.
- **Structure:**
  - The first line should be a concise summary of the file’s purpose.
  - Followed by a blank comment line and additional comments detailing context, usage, and any important considerations.
  - Include a revision history using date-stamped entries.

## 3. Imports and Module Organization

- **Rule:** Use ES Modules with explicit import paths. Avoid deep or ambiguous relative paths when possible.
- **Usage:** Always specify file extensions when importing local modules if required by your build system. Prefer explicit imports over wildcard imports to ensure clarity.
- **Example:**
  - Correct: `import { UserService } from './services/userService';`
  - Avoid: `import * as services from '../services';`

## 4. Coding Style and Functional Practices

- **Style:** Favor a functional programming style where applicable. Utilize immutable variables (`const`) over mutable ones (`let`) and apply arrow functions for concise syntax.
- **Best Practices:**
  - Keep functions short and focused. Aim for functions to do one thing.
  - Use array methods like `map`, `filter`, and `reduce` for data transformations.

## 5. Component and Function Length Limits

- **Rule:** Components and functions should ideally not exceed 100 lines of code to maintain readability and ease of testing.
- **Recommendation:** Break complex logic into smaller, reusable functions or components. If a file grows too large, consider refactoring by isolating logic into separate modules.

## 6. In-File and Co-Located Testing Guidelines

- **Testing:** Unit tests for components and functions should reside close to the implementation. For React components, consider co-locating tests with the component file using a `.test.tsx` suffix or within the same directory.
- **Guidelines:** Ensure tests are clear, cover both success and edge cases, and document usage where necessary.

## 7. No Unused or Redundant Code

- **Rule:** Regularly review files for unused variables, redundant code blocks, or legacy comments. Keeping a lean codebase is critical for maintainability.

## 8. Documentation and Comments

- **Directive:** Write clear and concise comments where necessary. However, avoid over-commenting obvious code. Use JSDoc style comments for public APIs to provide type and usage information.

---

# Example Ideal File: user-profile.tsx

```tsx
/**
 * UserProfile Component
 *
 * This component renders a user's profile information including avatar, name, and bio.
 *
 * Revision History:
 * - 2025-04-13T13:38:44Z @AI: Initial creation based on frontend standards.
 */

import React from 'react';

interface UserProfileProps {
  userName: string;
  avatarUrl: string;
  bio: string;
}

const UserProfile: React.FC<UserProfileProps> = ({ userName, avatarUrl, bio }) => {
  return (
    <div className="user-profile">
      <img src={avatarUrl} alt={`${userName}'s avatar`} />
      <h2>{userName}</h2>
      <p>{bio}</p>
    </div>
  );
};

export default UserProfile;
```

These guidelines are designed to ensure a consistent, maintainable, and high-quality codebase for the TypeScript frontend. Developers are encouraged to follow these standards and update them as needed to reflect evolving best practices.

IMPORTANT:
- **Application, Not Inclusion:** These guidelines describe *how* to write code. Do not copy the text of these guidelines into your Typescript source files. Apply the principles described here.