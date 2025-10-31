# Google OAuth2 Authentication Setup

This document explains how to set up Google OAuth2 authentication for the narrativ application.

## Overview

The application now supports Google OAuth2 authentication alongside traditional email/password login. Users can sign in with their Google accounts, and new users will be automatically created in the system.

## Google Cloud Console Setup

1. **Create a Google Cloud Project** (if you don't have one):
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select an existing one

2. **Enable the necessary APIs**:
   - Navigate to "APIs & Services" > "Library"
   - Search for and enable these APIs:
     - **Google OAuth2 API** (included by default)
     - **Google People API** (if you need additional profile information)
   - **Note**: The Google+ API mentioned in older documentation has been deprecated and is no longer required

3. **Create OAuth2 Credentials**:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth 2.0 Client IDs"
   - Select "Web application" as the application type
   - Add authorized redirect URIs:
     - For development: `http://localhost:8080/auth/google/callback`
     - For production: `https://yourdomain.com/auth/google/callback`

4. **Get Your Credentials**:
   - Note down the Client ID and Client Secret

## Environment Configuration

Add the following environment variables to your `.env` file:

```bash
# Google OAuth2 Configuration
GOOGLE_CLIENT_ID=your_google_client_id_here
GOOGLE_CLIENT_SECRET=your_google_client_secret_here
GOOGLE_REDIRECT_URL=http://localhost:8080/auth/google/callback
```

For production, update the `GOOGLE_REDIRECT_URL` to match your domain.

## API Endpoints

The following OAuth2 endpoints are available:

### `GET /auth/google`
Initiates the Google OAuth2 flow. Redirects users to Google's authorization server.

**Query Parameters:**
- `return_url` (required): Complete URL where user should be redirected after successful OAuth

**Example**: `GET /auth/google?return_url=https://app.narrativ.io/dashboard`

### `GET /auth/google/callback`
Handles the callback from Google. Processes the authorization code and creates/logs in the user.

**Query Parameters:**
- `code`: Authorization code from Google
- `state`: CSRF protection token with encoded return URL
- `error`: Error message (if authorization failed)

**Response:** Same format as regular login - returns JWT token and user information.

## Frontend Integration

To integrate with the frontend, add a "Sign in with Google" button that redirects to `/auth/google` with a return URL:

```javascript
// Example: Redirect to Google OAuth2 with return URL
const returnUrl = encodeURIComponent(window.location.href);
window.location.href = `/auth/google?return_url=${returnUrl}`;
```

The user will be redirected back to your specified return URL after authentication.

## Security Features

Our OAuth2 implementation includes comprehensive security measures:

### **CSRF Protection**
- Uses cryptographically secure state parameters with base64 encoding
- Each OAuth2 request includes a unique CSRF token that's validated on callback
- State parameter contains both CSRF token and return URL for secure transmission

### **URL Validation** 
- Return URLs are validated against allowed domains to prevent open redirect attacks
- Malformed or suspicious URLs are rejected with proper error messages

### **Input Sanitization**
- All OAuth2 parameters are validated and sanitized before processing
- Protection against SQL injection, XSS, and other injection attacks
- URL encoding/decoding handled securely

### **User Management**
- **Automatic User Creation**: New users are automatically created with their Google email
- **Existing User Login**: Existing users with matching emails are logged in automatically  
- **Secure Password Handling**: OAuth2 users get cryptographically secure placeholder password hashes
- **Account Security**: OAuth2-only users cannot use password-based login for enhanced security

### **Token Security**
- JWT tokens include user ID, admin status, and expiration
- Tokens are signed with server secret for tamper protection
- Proper token validation on all protected endpoints

## Troubleshooting

### Common Issues:

1. **"OAuth2 configuration error"**: 
   - Check that `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` are set correctly
   - Verify environment variables are loaded in your application

2. **"Invalid redirect URI"**: 
   - Ensure the redirect URI in Google Cloud Console matches your `GOOGLE_REDIRECT_URL` exactly
   - Check for trailing slashes and port number mismatches

3. **"Invalid CSRF token"** or state validation errors:
   - This usually happens if the OAuth2 flow is interrupted or cookies are blocked
   - Clear browser cookies and try again

4. **"return_url parameter is required"**:
   - The `/auth/google` endpoint requires a `return_url` parameter
   - Ensure your frontend is passing this parameter correctly

### Development vs Production:

- **Development**: Use `http://localhost:8080/auth/google/callback` (note: port 8080, not 8081)
- **Production**: Use your actual domain with HTTPS

### Logs

Check the application logs for detailed error messages. OAuth2 errors are logged with context about what went wrong:

```
INFO: Redirecting to Google OAuth2 with return URL: https://app.narrativ.io/dashboard
WARN: OAuth2 error received: access_denied  
ERROR: Failed to base64 decode state: Invalid symbol 95, offset 5
```

## Migration Plan Integration

This OAuth2 implementation supports both applications (narrativ and realestate) as requested in the migration plan. The system automatically detects the context based on the return URL and redirects users to the appropriate branded interface after authentication.

**Real Estate Context**: When users access `/real-estate` routes, they get the real estate branded login page with Google OAuth2 support, and successful authentication redirects back to the real estate interface.

**Narrativ Context**: Standard application routes use the narrativ branding and redirect appropriately after OAuth2 completion. 