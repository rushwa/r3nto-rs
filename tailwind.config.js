/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    // Scan all Rust files for Tailwind classes
    files: [
      "./crates/**/src/**/*.rs",
      "./crates/**/src/**/*.html",
      "./index.html",
    ],
    // Extract class names from Rust strings
    extract: {
      rs: (content) => {
        // Match class: "..." and class: "..." patterns
        const matches = content.match(/class:\s*"([^"]+)"/g) || [];
        return matches.map(m => m.replace(/class:\s*"/, '').replace(/"/, ''));
      }
    }
  },
  theme: {
    extend: {
      colors: {
        // Add your custom brand colors here
        rento: {
          blue: '#2563eb',
          green: '#10b981',
        }
      }
    },
  },
  plugins: [],
}
