export const touchingDetector = { isTouching: false };

window.addEventListener("touchstart", () => {
    touchingDetector.isTouching = true;
});
window.addEventListener("touchend", () => {
    setTimeout(() => {
        touchingDetector.isTouching = false;
    }, 100);
});
