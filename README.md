# Bondi - lock-free single producer multi producer ring buffer

Bondi is yet another attempt of producing a lock-free, bounded, single-producer, multi-consumer data structure.

Note: this is an experimental project. Expect things to break, and change.

## Usage

```rust
    // initialize a writer and two readers
    // send 100 `Message`s, and receive them from different threads
    struct Message(usize)

    fn main() {
        let bondi = Bondi::<Message>::new(100);
        let writer = bondi.get_tx().unwrap();
        let reader = bondi.get_rx().unwrap();
        let reader2 = bondi.get_rx().unwrap();

        std::thread::spawn(move || {
            for i in 0..100 {
                writer.write(Message(i));
            }
        });

        std::thread::spawn(move || {
            let _ = reader.read();
        });

        std::thread::spawn(move || {
            let _ = reader2.read();
        }).join().unwrap();
    }

```

## Inspiration sources

[Disruptor](https://github.com/LMAX-Exchange/disruptor)

[Bus](https://github.com/jonhoo/bus/)

[Turbine](https://github.com/polyfractal/Turbine)
