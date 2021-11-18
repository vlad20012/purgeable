use bytesize::ByteSize;
use purgeable::{NonPurgeableBox, PurgeableBox};
use std::io::BufRead;
use std::ops::Add;

struct MaybePurgedBox<T: ?Sized> {
    inner: Option<PurgeableBox<T>>,
    size: usize,
}

impl<T: ?Sized> MaybePurgedBox<T> {
    //noinspection RsSelfConvention
    fn is_purged(&mut self) -> bool {
        let inner = self.inner.take();
        match inner.and_then(|b| b.lock().ok()) {
            None => true,
            Some(b) => {
                self.inner = Some(NonPurgeableBox::unlock(b));
                false
            }
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut pgable = Vec::<MaybePurgedBox<[u8]>>::new();
    let mut boxes = Vec::<Box<[u8]>>::new();
    loop {
        let stdin = std::io::stdin();
        let mut inp = stdin.lock();
        let mut line = String::new();
        inp.read_line(&mut line)?;
        let line = line.trim();

        if line == "q" || line == "quit" || line == "e" || line == "exit" {
            return Ok(());
        }

        if line.starts_with("repeat ") {
            let line = &line["repeat ".len()..];
            let space_idx = line.find(" ").unwrap();
            let num = line[..space_idx].parse::<i32>().unwrap();
            for _ in 0..num {
                perform_command(&mut pgable, &mut boxes, &line[space_idx.add(1)..])
            }
        } else {
            perform_command(&mut pgable, &mut boxes, &line)
        }
    }
}

fn perform_command(pgable: &mut Vec<MaybePurgedBox<[u8]>>, boxes: &mut Vec<Box<[u8]>>, line: &str) {
    if line.starts_with("purgeable ") || line.starts_with("p ") {
        let size = parse_size(&line[line.find(" ").unwrap().add(1)..]);
        let b = NonPurgeableBox::<[u8]>::new_filled_slice(0, size);
        let b = NonPurgeableBox::unlock(b);

        pgable.push(MaybePurgedBox {
            inner: Some(b),
            size,
        });

        let size = ByteSize::b(size as u64).to_string_as(true);
        println!("Allocated {} of purgeable memory", size);
        print_stats(pgable, &boxes);
    }

    if line.starts_with("alloc ") || line.starts_with("a ") {
        let size = parse_size(&line[line.find(" ").unwrap().add(1)..]);
        let b = vec![1u8; size];

        boxes.push(b.into_boxed_slice());

        let size = ByteSize::b(size as u64).to_string_as(true);
        println!("Allocated {} of non-purgeable memory", size);
        print_stats(pgable, &boxes);
    }
}

fn print_stats(pgable: &mut Vec<MaybePurgedBox<[u8]>>, boxes: &Vec<Box<[u8]>>) {
    let total = ByteSize::b(pgable.iter().map(|b| b.size as u64).sum());
    let purged = ByteSize::b(
        pgable
            .iter_mut()
            .map(|b| if b.is_purged() { b.size as u64 } else { 0 })
            .sum(),
    );
    let total_b = ByteSize::b(boxes.iter().map(|b| b.len() as u64).sum());
    println!(
        "Total purgeable: {} ({} purged). Total non-purgeable: {}",
        total.to_string_as(true),
        purged.to_string_as(true),
        total_b.to_string_as(true)
    );
}

fn parse_size(size_str: &str) -> usize {
    let modifier = match size_str.bytes().last().unwrap() {
        b'k' | b'K' => 1024,
        b'm' | b'M' => 1024 * 1024,
        b'g' | b'G' => 1024 * 1024 * 1024,
        _ => 1,
    };
    let size_str = if modifier == 1 {
        size_str
    } else {
        &size_str[..size_str.len() - 1]
    };
    let size = size_str.parse::<usize>().unwrap() * modifier;
    size
}
