macro_rules! consume_peeked {
    ($cursor:ident, $mapper:ident, $peeked:ident) => {
        $cursor.apply($peeked);
        $mapper.next();
    };
}

pub(crate) use consume_peeked;
