try {
  const theme = localStorage.getItem("trellis.console.theme");
  document.documentElement.setAttribute(
    "data-theme",
    theme === "trellis-dark" ? "trellis-dark" : "trellis",
  );
} catch (_) {
  document.documentElement.setAttribute("data-theme", "trellis");
}
