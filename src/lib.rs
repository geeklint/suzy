extern crate drying_paint;

pub mod dims;
pub mod graphics;
pub mod interaction;
pub mod platform;
pub mod widget;

/// ```rust,no_run
/// struct NoChildren {
/// }
///
/// impl suzy::widget::WidgetData for NoChildren {
///     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
///     suzy::children!();
/// }
/// ```
///
/// ```rust,no_run
/// # struct NoChildren {
/// # }
/// # impl suzy::widget::WidgetData for NoChildren {
/// #     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
/// #     suzy::children!();
/// # }
/// struct TwoChildren {
///     left: suzy::widget::Widget<NoChildren>,
///     right: suzy::widget::Widget<NoChildren>,
/// }
///
/// impl suzy::widget::WidgetData for TwoChildren {
///     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
///     suzy::children!(left => NoChildren, right => NoChildren);
/// }
/// ```
#[macro_export]
macro_rules! children {
    ( $($x:ident => $t:ty),+ ) => { $crate::children!($($x => $t,)*); };
    () => {
        type ChildA = ();
        type ChildB = ();
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildren::Zero }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildrenMut::Zero }
    };
    ($a:ident => $at:ty,) => {
        type ChildA = $at;
        type ChildB = ();
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildren::One(&self.$a) }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildrenMut::One(&mut self.$a) }
    };
    ($a:ident => $at:ty, $b:ident => $bt:ty,) => {
        type ChildA = $at;
        type ChildB = $bt;
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildren::Two(&self.$a, &self.$b) }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildrenMut::Two(
            &mut self.$a, &mut self.$b) }
    };
    ($a:ident => $at:ty, $b:ident => $bt:ty, $c:ident => $ct:ty,) => {
        type ChildA = $at;
        type ChildB = $bt;
        type ChildC = $ct;
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   children_expr!(&self.$a, &self.$b, &self.$c) }
        { $crate::widget::children::WidgetChildren::Three(
            &self.$a, &self.$b, &self.$c) }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildrenMut::Three(
            &mut self.$a, &mut self.$b, &mut self.$c) }
    };
    (   $a:ident => $at:ty,
        $b:ident => $bt:ty,
        $c:ident => $ct:ty,
        $d:ident => $dt:ty,
    ) => {
        type ChildA = $at;
        type ChildB = $bt;
        type ChildC = $ct;
        type ChildD = $dt;
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildren::Four(
            &self.$a, &self.$b, &self.$c, &self.$d) }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        { $crate::widget::children::WidgetChildrenMut::Four(
            &mut self.$a, &mut self.$b, &mut self.$c, &mut self.$d) }
    };
    ( $($x:ident => $xt:ty,)* ) => {
        type ChildA = ();
        type ChildB = ();
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec![ $(self.$x.proxy(),)* ];
            $crate::widget::children::WidgetChildren::other(list)
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec! [$(self.$x.proxy_mut(),)* ];
            $crate::widget::children::WidgetChildrenMut::other(list)
        }
    };
}


#[cfg(test)]
mod tests {
    struct NoChildren {
    }
    impl super::widget::WidgetData for NoChildren {
        fn init(init: &mut super::widget::WidgetInit<Self>) {}
        super::children!();
    }
    struct ManyChildren {
        a: super::widget::Widget<NoChildren>,
        b: super::widget::Widget<NoChildren>,
        c: super::widget::Widget<NoChildren>,
        d: super::widget::Widget<NoChildren>,
        e: super::widget::Widget<NoChildren>,
        f: super::widget::Widget<NoChildren>,
    }
    impl super::widget::WidgetData for ManyChildren {
        fn init(init: &mut super::widget::WidgetInit<Self>) {}
        super::children!(
            a => NoChildren,
            b => NoChildren,
            c => NoChildren,
            d => NoChildren,
            e => NoChildren,
            f => NoChildren,
        );
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
