try {
  var stored = localStorage.getItem("frameforge.colorScheme");
  if (stored === "light" || stored === "dark") {
    var meta = document.querySelector('meta[name="color-scheme"]');
    if (meta) {
      meta.setAttribute("content", stored);
    }
  }
} catch (e) {}

