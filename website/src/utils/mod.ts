export function debounceEventHandler<Ev extends Event>(
  handler: (ev: Ev) => void,
) {
  let lastEv: Ev | null = null;
  let handling = false;
  return (ev: Ev) => {
    lastEv = ev;
    if (handling) {
      requestAnimationFrame(() => {
        if (lastEv === ev) {
          handler(ev);
        }
      });
    } else {
      handling = true;
      requestAnimationFrame(() => {
        handler(ev);
        handling = false;
      });
    }
  };
}
