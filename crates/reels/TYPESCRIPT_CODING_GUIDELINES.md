### ALWAYS INCLUDE ME IN THE CONTEXT!!! ###

# TypeScript & React Coding Guidelines

**Version:** 1.0  
**Date:** 2025-04-21

---

## 1. Introduction

This document defines TypeScript and React best practices for building maintainable, performant, and type-safe frontend applications in this monorepo.

---

## 2. Guiding Principles

- **Strict Typing Everywhere:** Enable `strict` mode in `tsconfig.json`. Disallow `any` and `noImplicitAny`.
- **Functional Components & Hooks:** Favor React functional components and hooks over class components.
- **Composition Over Inheritance:** Build UIs with reusable, small components and custom hooks.
- **Declarative UI:** Keep JSX clear and descriptive, minimize imperative code within components.
- **Single Responsibility:** Each component or hook should handle one concern.
- **Short Files & Components:** Aim for ≤200 lines per file, ≤30 lines per function.
- **Automated Formatting & Linting:** Use Prettier and ESLint with TypeScript-React plugins.
- **Comprehensive Testing:** Unit test logic with Vitest/Jest, test components with React Testing Library, and E2E with Playwright.
- **AI-Assisted Development:** Use AI tools for boilerplate, refactorings, and test skeletons—but review all generated code.

---

## 3. Project Structure

```
frontend/
├── src/
│   ├── components/        # Reusable UI components
│   ├── hooks/             # Custom React hooks
│   ├── utils/             # Pure utility functions (stateless)
│   ├── pages/             # Page-level components/routes
│   ├── api/               # API client and service functions
│   ├── types/             # Shared TypeScript type definitions
│   └── index.tsx          # App entry point
├── tests/
│   └── component/         # Component integration tests
└── tsconfig.json
```

---

## 4. TypeScript Standards

- **No Implicit Any:** Enforce in `tsconfig.json`.
- **Strict Null Checks:** Enable to catch undefined/null errors early.
- **No `@ts-ignore`:** Avoid suppressing errors; fix type issues directly.
- **Type Aliases & Interfaces:** Use `type` for unions & primitives, `interface` for object shapes and props.
- **Discriminated Unions:** Model variant props or states clearly.
- **Utility Types:** Leverage `Partial`, `Pick`, `Record`, etc., for DRY definitions.
- **Mapped Types & Generics:** Use generics for reusable, type-safe utilities.

---

## 5. React & JSX Guidelines

- **Component Naming:** `PascalCase` for components, file name matches component name.
- **Prop Typing:** Define props via interfaces; always type component props.
- **Default Props:** Use default parameters or `Partial` prop interfaces.
- **Destructure Props:** At function signature for clarity.
- **Event Handlers:** Use explicit React types, e.g., `React.MouseEvent<HTMLButtonElement>`.
- **JSX Formatting:** One prop per line when >3 props; wrap long JSX in parentheses.
- **Hooks Rules:** Follow the Rules of Hooks; place hooks at top level only.
- **State & Effects:** Keep `useEffect` dependencies accurate; extract complex effects into custom hooks.

---

## 6. Styling & CSS

- **CSS Modules or Styled Components:** Encapsulate styles per component.
- **Class Name Convention:** `kebab-case` for CSS modules, `CamelCase` for styled-components.
- **Theming:** Use a centralized theme object with TypeScript types.
- **Responsive Design:** Leverage CSS variables and media queries with typed breakpoints.

---

## 7. Testing Strategy

- **Unit Tests:** Use Vitest or Jest for pure logic and small components.
- **Component Tests:** React Testing Library for interaction and rendering assertions.
- **Mocking:** Mock API calls with `msw` (Mock Service Worker).
- **E2E Tests:** Playwright for user flows.
- **Coverage:** Aim for ≥80% coverage on critical modules.

---

## 8. Tooling & Automation

- **Linting:** ESLint with `@typescript-eslint` and `eslint-plugin-react`.
- **Formatting:** Prettier with project config; run on save and pre-commit.
- **HMR:** Use Vite or Webpack for fast refresh in development.
- **CI Checks:** Integrate lint, format check, and tests in GitHub Actions pipeline.

---

## 9. Version Control & Branching

- **Feature Branches:** One feature per branch; merge via PR with code review.
- **Commit Messages:** Use Conventional Commits (e.g., `feat`, `fix`, `refactor`).
- **Rebase & Squash:** Maintain a clean history.

---

## 10. Continuous Improvement

- Review these guidelines regularly.
- Add project-specific patterns under `types/` or `hooks/` as needed.
- Incorporate feedback from audits and testing failures.
