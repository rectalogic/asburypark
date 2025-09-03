const root = document.documentElement;
const daySelect = document.getElementById("day");
const hourSelect = document.getElementById("hour");

root.dataset.selectedDayhour = `${daySelect.value}-${hourSelect.value}`;
window.addEventListener("change", () => {
  root.dataset.selectedDayhour = `${daySelect.value}-${hourSelect.value}`;
});
