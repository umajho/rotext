interface Obeserver {
  subscribe(cb: () => void): void;
  unsubscribe(cb: () => void): void;
}

export class ElementLayoutChangeObserver implements Obeserver {
  private readonly _subscribers = new Set<() => void>();
  public readonly scopes: {
    resize?: true;
  };

  private _resizeObserver: ResizeObserver | null = null;

  private readonly _notify = () => {
    [...this._subscribers].forEach((cb) => cb());
  };

  constructor(
    public readonly targetElement: HTMLElement,
    scopes: ElementLayoutChangeObserver["scopes"],
  ) {
    this.scopes = Object.freeze(scopes);
  }

  subscribe(cb: () => void): void {
    const oldSize = this._subscribers.size;
    this._subscribers.add(cb);
    if (oldSize === 0) {
      this._start();
    }
  }

  private _start() {
    if (this.scopes.resize) {
      this._resizeObserver = new ResizeObserver(this._notify);
      this._resizeObserver.observe(this.targetElement);
    }
  }

  unsubscribe(cb: () => void): void {
    const oldSize = this._subscribers.size;
    this._subscribers.delete(cb);
    if (oldSize === 1 && this._subscribers.size === 0) {
      this._stop();
    }
  }

  private _stop() {
    if (this.scopes.resize) {
      this._resizeObserver!.disconnect();
    }
  }
}

export class MultiObserver implements Obeserver {
  constructor(private _observers: Obeserver[]) {}

  subscribe(cb: () => void): void {
    this._observers.forEach((o) => o.subscribe(cb));
  }

  unsubscribe(cb: () => void): void {
    this._observers.forEach((o) => o.unsubscribe(cb));
  }
}
