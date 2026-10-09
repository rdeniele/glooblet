// Glooblet landing page demo logic. Everything here is genuinely
// interactive: real cm/360 math (same formula as src-tauri/src/calc.rs)
// and real audio playback of the actual bundled click sounds. Nothing here
// touches real Windows settings; the demo panels say so.

(() => {
  "use strict";

  // --- Tab switching -------------------------------------------------------

  const tabs = document.querySelectorAll(".demo-tab");
  const panels = document.querySelectorAll(".demo-panel");

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const target = tab.dataset.tab;

      tabs.forEach((t) => {
        const active = t === tab;
        t.classList.toggle("is-active", active);
        t.setAttribute("aria-selected", String(active));
      });

      panels.forEach((panel) => {
        const active = panel.dataset.panel === target;
        panel.classList.toggle("is-active", active);
        panel.hidden = !active;
      });
    });
  });

  // --- Mouse Settings demo ---------------------------------------------------

  const speedInput = document.getElementById("demo-speed");
  const speedOut = document.getElementById("demo-speed-out");
  const accelToggle = document.getElementById("demo-accel");
  const mouseReadout = document.getElementById("demo-mouse-readout");

  let accelOn = false;

  function renderMouseReadout() {
    const speed = speedInput.value;
    speedOut.textContent = `${speed} / 20`;
    mouseReadout.textContent = accelOn
      ? `Speed ${speed}, Enhance pointer precision ON: acceleration applied on fast swipes.`
      : `Speed ${speed}, acceleration off: consistent 1:1 movement, the competitive default.`;
  }

  speedInput.addEventListener("input", renderMouseReadout);
  accelToggle.addEventListener("click", () => {
    accelOn = !accelOn;
    accelToggle.setAttribute("aria-checked", String(accelOn));
    renderMouseReadout();
  });
  renderMouseReadout();

  // --- Game Profiles demo: real cm/360 math -------------------------------

  const gameSelect = document.getElementById("demo-game");
  const dpiInput = document.getElementById("demo-dpi");
  const sensInput = document.getElementById("demo-sens");
  const cm360Readout = document.getElementById("demo-cm360-readout");

  function renderCm360() {
    const dpi = Number(dpiInput.value);
    const sens = Number(sensInput.value);
    const yaw = Number(gameSelect.value);
    const gameName = gameSelect.options[gameSelect.selectedIndex].textContent;
    const confidence = gameSelect.options[gameSelect.selectedIndex].dataset.confidence;

    if (!dpi || dpi <= 0 || !sens || sens <= 0) {
      cm360Readout.textContent = "Enter a positive DPI and sensitivity.";
      return;
    }

    // Same formula as calc::cm_per_360 in the Rust backend:
    // cm_per_360 = (2.54 * 360) / (dpi * sensitivity * yaw_per_count)
    const cm = (2.54 * 360) / (dpi * sens * yaw);
    const confidenceLabel = confidence === "high" ? "well-documented constant" : "community-sourced constant";

    cm360Readout.textContent = `${cm.toFixed(1)} cm per 360° turn in ${gameName} (${confidenceLabel}, yaw ${yaw}).`;
  }

  [gameSelect, dpiInput, sensInput].forEach((el) => el.addEventListener("input", renderCm360));
  renderCm360();

  // --- Click Studio demo: real audio playback -----------------------------

  const soundButtons = document.querySelectorAll(".demo-sound");
  let currentAudio = null;

  soundButtons.forEach((button) => {
    button.addEventListener("click", () => {
      if (currentAudio) {
        currentAudio.pause();
        currentAudio.currentTime = 0;
      }
      soundButtons.forEach((b) => b.classList.remove("is-playing"));

      const audio = new Audio(button.dataset.src);
      currentAudio = audio;
      button.classList.add("is-playing");
      audio.addEventListener("ended", () => button.classList.remove("is-playing"));
      audio.play().catch(() => {
        button.classList.remove("is-playing");
      });
    });
  });

  // Note: the download button's href is set directly in the markup (a raw
  // GitHub content link to the committed portable .exe). A client-side
  // fetch() probe to check the file exists was tried here and removed:
  // GitHub doesn't send CORS headers on that endpoint, so the check always
  // fails even when the file is right there; a plain <a href> download
  // doesn't need CORS at all, so it just works without the probe.
})();
