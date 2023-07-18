export function debounceEventHandler<Ev extends Event, T>(
  handler: (ev: Ev, data: T) => void,
) {
  let lastEv: Ev | null = null;
  let handling = false;
  return (ev: Ev, data: T) => {
    lastEv = ev;
    if (handling) {
      requestAnimationFrame(() => {
        if (lastEv === ev) {
          handler(ev, data);
        }
      });
    } else {
      handling = true;
      requestAnimationFrame(() => {
        handler(ev, data);
        handling = false;
      });
    }
  };
}
