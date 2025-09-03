const root = document.documentElement;
const daySelect = document.getElementById("day");
const hourSelect = document.getElementById("hour");

window.addEventListener("change", () => {
  root.dataset.selectedHour = hourSelect.value;
  root.dataset.selectedDay = daySelect.value;
  root.dataset.selectedDayhour = `${daySelect.value}-${hourSelect.value}`;
});
