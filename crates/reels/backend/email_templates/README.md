# Email Templates

This directory contains templates for emails sent by the application.

## Purpose

These templates define the structure and content of emails for various user interactions, such as:
- Welcome emails for new users (`welcome_email.html`)
- Email verification messages (`verification_email.html`)
- Password reset instructions (`password_reset_email.html`)

## Templating Engine

The templates use a syntax similar to Jinja2 or Tera (commonly used in Rust web frameworks like Actix-web with the `tera` crate). Key features observed:

- **Inheritance:** Using `{% extends "base.html" %}` to inherit from a base layout. Ensure a `base.html` exists in a location accessible by the template engine (e.g., this directory or a configured parent).
- **Blocks:** Defining content sections using `{% block content %}` ... `{% endblock %}` which override corresponding blocks in the base template.
- **Variables:** Injecting dynamic data using double curly braces `{{ variable_name }}`.

## Usage

### Variables

Variables are placeholders replaced with actual data when the email is generated. Examples found in the templates include:

- `{{ user_name }}`: The name of the recipient.
- `{{ verification_token }}`: A token for email verification.
- `{{ reset_link }}`: A URL for resetting the password.

Ensure the backend code provides the necessary context data (e.g., a struct or map) matching these variable names when rendering a template.

### Integration (Example with Rust/Tera)

To use these templates in the backend:

1.  **Add Dependency:** Add `tera` to your `Cargo.toml`.
2.  **Initialize Engine:** Create a `Tera` instance, usually pointing to this directory (e.g., `Tera::new("backend/email_templates/**/*.html")?`). This might be done once at application startup and stored in application state (like Actix-web's `Data`).
3.  **Prepare Context:** Create a `tera::Context` and insert the required variables (e.g., `context.insert("user_name", &user.name);`).
4.  **Render Template:** Call `tera.render("welcome_email.html", &context)` to get the HTML string.
5.  **Send Email:** Use an email sending library (like `lettre`) or service to deliver the generated email content.

## Customization

- Modify existing templates to match application branding and specific wording requirements.
- Add new templates for other email types as needed.
- Create or update the `base.html` template to define common HTML structure, headers, footers, and basic CSS styling for all emails.