function identity<T>(arg: T): T {
  return arg;
}

interface Box<T> {
  value: T;
}

const box: Box<number> = { value: 42 };
