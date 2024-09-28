// hljs.highlightAll();
var themeToggle = document.getElementById("theme-toggle");

if (themeToggle) {
  themeToggle.setAttribute("title", "تغيير المظهر");
  themeToggle.setAttribute("aria-label", "تغيير المظهر");
}

var sidebarToggle = document.getElementById("sidebar-toggle");

if (sidebarToggle) {
  sidebarToggle.setAttribute("title", "إظهار/إخفاء جدول المحتويات");
  sidebarToggle.setAttribute("aria-label", "إظهار/إخفاء جدول المحتويات");
}

document.getElementById("light").textContent = "فاتح";
document.getElementById("rust").textContent = "بني";
document.getElementById("coal").textContent = "كُحلي";
document.getElementById("navy").textContent = "نيلي";
document.getElementById("ayu").textContent = "غامق";

const alerts = [
  ["ملحوظة", "note"],
  ["إرشاد", "tip"],
  ["مهم", "important"],
  ["تحذير", "warning"],
  ["تنبيه", "caution"],
];

for (alert of alerts) {
  document.querySelectorAll(`.mdbook-alerts-${alert[0]}`).forEach((e) => {
    e.classList.replace(
      `mdbook-alerts-${alert[0]}`,
      `mdbook-alerts-${alert[1]}`
    );
  });
}

document.querySelectorAll(".mdbook-alerts").forEach((e) => {
  e.style.borderLeft = "none";
});
