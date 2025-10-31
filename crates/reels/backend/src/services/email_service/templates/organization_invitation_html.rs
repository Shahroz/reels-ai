//! HTML template for organization invitation emails.
//!
//! This template is used when an admin adds a user to an organization.
//! It uses table-based layout with inline styles for maximum email client compatibility.
//! The template expects 4 string interpolations: organization_name (with <strong> tags),
//! invitation_link (3 times for button, fallback link href, and fallback link text).

pub const ORGANIZATION_INVITATION_HTML_TEMPLATE: &str = r##"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional //EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <meta charset="utf-8">
    <meta http-equiv="Content-Type" content="text/html; charset=UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="margin:0; padding:0; font-family:Arial, sans-serif; font-size:15px; color:#23496d; background-color:#F9FAF5;">
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:#F9FAF5; border-collapse:collapse;">
        <tr>
            <td align="center" style="padding:20px 10px;">
                
                <!-- Logo Section -->
                <table role="presentation" width="600" cellpadding="0" cellspacing="0" style="background-color:#FFFFFF; max-width:600px; border-collapse:collapse;">
                    <tr>
                        <td align="center" style="padding:10px 20px;">
                            <img src="https://communication.bounti.ai/hs-fs/hubfs/Bounti_Multicolor_lockup.png?width=200" alt="Bounti" width="100" style="width:100px; height:auto; display:block; border:0; outline:none;">
                        </td>
                    </tr>
                </table>
                
                <!-- Content Section -->
                <table role="presentation" width="600" cellpadding="0" cellspacing="0" style="background-color:#FFFFFF; max-width:600px; border-collapse:collapse;">
                    <tr>
                        <td style="padding:30px 20px; font-family:Arial, sans-serif; font-size:15px; color:#23496d;">
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:0 0 15px 0;">
                                You've been added to your team's organization {}. We're excited to have you. Click the button below to join your fellow agents.
                            </p>
                            
                            <!-- Button -->
                            <table role="presentation" cellpadding="0" cellspacing="0" style="margin:30px auto; border-collapse:separate;">
                                <tr>
                                    <td align="center" bgcolor="#EE5936" style="border-radius:8px; background-color:#EE5936;">
                                        <a href="{}" style="display:inline-block; padding:12px 18px; font-family:Arial, sans-serif; font-size:16px; color:#FFFFFF; text-decoration:none; font-weight:bold;">Join Your Team</a>
                                    </td>
                                </tr>
                            </table>
                            
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:30px 0 0 0;">
                                If that button doesn't work use this link:
                            </p>
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:15px 0;">
                                <a href="{}" style="color:#00a4bd; text-decoration:underline;">{}</a>
                            </p>
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:15px 0;">
                                &nbsp;
                            </p>
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:15px 0;">
                                Thanks,
                            </p>
                            <p style="font-family:Arial, sans-serif; font-size:15px; color:#23496d; line-height:175%; margin:0;">
                                <br>The Bounti team
                            </p>
                        </td>
                    </tr>
                </table>
                
                <!-- Footer Section -->
                <table role="presentation" width="600" cellpadding="0" cellspacing="0" style="max-width:600px; border-collapse:collapse;">
                    <tr>
                        <td align="center" style="padding:20px; font-family:Arial, sans-serif; font-size:12px; color:#23496d; line-height:135%;">
                            <p style="margin:0;">
                                Bounti Labs, 2099 Gateway Place, Suite 560, San Jose, CA 95110, United States
                            </p>
                        </td>
                    </tr>
                </table>
                
            </td>
        </tr>
    </table>
</body>
</html>"##;

