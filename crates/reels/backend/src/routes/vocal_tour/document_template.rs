//! HTML document template for vocal tour documents.
//!
//! Contains the standard HTML structure with styling for vocal tour property descriptions.
//! The template includes placeholders for title and body content.

/// Standard HTML template for vocal tour documents.
/// 
/// Contains placeholders:
/// - `{title}`: Property title
/// - `{body}`: Main content body including property description, photo gallery, and transcript
pub const VOCAL_TOUR_DOCUMENT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap');

        body {
            background-color: #fbeee7; 
            font-family: 'Roboto', sans-serif;
            color: #4D4D4D; 
            line-height: 1.6;
            margin: 0;
            padding: 20px;
        }

        .container {
            background-color: #ffffff; 
            max-width: 800px;
            margin: 20px auto;
            padding: 30px 40px;
            border-radius: 16px; 
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.08);
        }

        h1, h2, h3 {
            color: #D85836; 
            font-weight: 700;
        }

        h1 {
            text-align: center;
            font-size: 2.5em;
            margin-bottom: 20px;
        }

        h2 {
            font-size: 1.8em;
            border-bottom: 2px solid #f0f0f0;
            padding-bottom: 10px;
            margin-top: 30px;
            margin-bottom: 20px;
        }

        h3 {
            font-size: 1.4em;
            margin-top: 30px;
            margin-bottom: 15px;
        }
        
        p {
            margin-bottom: 1em;
        }

        strong, b {
             color: #4D4D4D; 
             font-weight: 700;
        }

        a {
            color: #0056b3; 
            text-decoration: none;
            font-weight: bold;
        }

        a:hover {
            text-decoration: underline;
            color: #003d80;
        }
        
        a b {
            color: inherit; 
        }

        ul {
            list-style-type: disc;
            padding-left: 25px;
        }

        li {
            margin-bottom: 12px;
            padding-left: 5px;
        }
        
        .transcript {
            background-color: #fdfdfd;
            border-left: 4px solid #FF6D00; 
            padding: 20px;
            margin-top: 25px;
            border-radius: 0 8px 8px 0;
        }
        
        .transcript p {
            margin: 0;
            font-style: italic;
            color: #555;
        }

    </style>
</head>
<body>
<div class="container">
{body}
</div>
</body>
</html>"#; 