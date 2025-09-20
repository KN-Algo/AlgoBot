#[macro_export]
macro_rules! map_properties {
    ($from:ident.$from_field:ident, $($prop: expr => $struct:ident.$field:ident $(;with $var:ident { $($closure:tt)* })?,)+) => {
        for prop in $from.$from_field {
            match prop.name.as_str() {
                $($prop => if let Some(p) = prop.value { map_properties!(@handle p $struct $field $(,$var, $($closure)* )?); })+
                _ => (),
            }
        }
    };

    (@handle $prop:ident $struct:ident $field:ident) => {
        $struct.$field = $prop
    };

    (@handle $prop:ident $struct:ident $field:ident, $var:ident, $($closure:tt)*) => {{
            let $var = $prop;
            $struct.$field = $($closure)*;
    }};
}
