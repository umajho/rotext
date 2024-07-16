macro_rules! consume_peeked {
    ($context:expr, $peeked:expr) => {
        $context.cursor.apply($peeked);
        $context.mapper.next();
    };
}

pub(crate) use consume_peeked;
