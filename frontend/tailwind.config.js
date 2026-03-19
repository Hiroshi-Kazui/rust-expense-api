/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Noto Sans JP", "sans-serif"],
      },
      colors: {
        primary: "#3B82F6",
        "primary-hover": "#2563EB",
      },
    },
  },
  plugins: [],
};
