// Make each route module public so they can be used elsewhere in the project

pub mod get;        // Handles GET requests (root and /get)
pub mod post;       // Handles POST requests (/post)
pub mod put;        // Handles PUT requests (/put)
pub mod patch;      // Handles PATCH requests (/patch)
pub mod delete;     // Handles DELETE requests (/delete)
pub mod options;    // Handles OPTIONS requests (/options)
pub mod status;     // Handles dynamic status code responses (/status/:code)
pub mod anything;   // Handles dynamic echoing of any request method (/anything)
pub mod healthz; 
