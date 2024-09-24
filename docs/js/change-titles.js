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
