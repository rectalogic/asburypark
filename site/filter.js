const sheet = new CSSStyleSheet();
document.adoptedStyleSheets.push(sheet);
window.addEventListener("change", () => {
  const day = document.getElementById("day").value;
  const hour = document.getElementById("hour").value;
  if (day === "" && hour === "") {
    sheet.replaceSync("");
  } else if (day !== "" && hour === "") {
    sheet.replaceSync(`
        tr.restaurant:not([data-days~="${day}"]) { display: none; }
        time.dayhour:not([data-days~="${day}"]) {
          background: var(--pico-del-color);
          text-decoration: line-through;
        }
      `);
  } else if (day === "" && hour !== "") {
    sheet.replaceSync(`
        tr.restaurant:not([data-hours~="${hour}"]) { display: none; }
        time.dayhour:not([data-hours~="${hour}"])  {
          background: var(--pico-del-color);
          text-decoration: line-through;
        }
      `);
  } else if (day !== "" && hour !== "") {
    sheet.replaceSync(`
        tr.restaurant:not([data-daytimes~="${day}-${hour}"]) { display: none; }
        time.dayhour:not([data-daytimes~="${day}-${hour}"])  {
          background: var(--pico-del-color);
          text-decoration: line-through;
        }
      `);
  }
});
