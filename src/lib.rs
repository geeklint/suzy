#[cfg(feature = "gag")]
extern crate gag;
#[cfg(feature = "lazy_static")]
extern crate lazy_static;
#[cfg(feature = "libc")]
extern crate libc;
#[cfg(feature = "gl")]
extern crate gl;
#[cfg(feature = "sdl2")]
extern crate sdl2;
#[cfg(feature = "widestring")]
extern crate widestring;

extern crate drying_paint;

pub mod app;
pub mod dims;
pub mod graphics;
pub mod interaction;
pub mod math;
pub mod units;
pub mod widget;
pub mod window;
pub(crate) mod platform;

/// ```rust,no_run
/// struct NoChildren {
/// }
///
/// impl suzy::widget::WidgetData for NoChildren {
///     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
///     suzy::children!();
///     type Graphic = ();
///     type GraphicAfter = ();
///     fn graphic(&self) -> &() { &() }
///     fn graphic_after(&self) -> &() { &() }
/// }
/// ```
///
/// ```rust,no_run
/// # struct NoChildren {
/// # }
/// # impl suzy::widget::WidgetData for NoChildren {
/// #     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
/// #     suzy::children!();
/// #     type Graphic = ();
/// #     type GraphicAfter = ();
/// #     fn graphic(&self) -> &() { &() }
/// #     fn graphic_after(&self) -> &() { &() }
/// # }
/// struct TwoChildren {
///     left: suzy::widget::Widget<NoChildren>,
///     right: suzy::widget::Widget<NoChildren>,
/// }
///
/// impl suzy::widget::WidgetData for TwoChildren {
///     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
///     suzy::children!(left => NoChildren, right => NoChildren);
///     type Graphic = ();
///     type GraphicAfter = ();
///     fn graphic(&self) -> &() { &() }
///     fn graphic_after(&self) -> &() { &() }
/// }
/// ```
///
/// ```rust,no_run
/// # struct NoChildren {
/// # }
/// # impl suzy::widget::WidgetData for NoChildren {
/// #     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
/// #     suzy::children!();
/// #     type Graphic = ();
/// #     type GraphicAfter = ();
/// #     fn graphic(&self) -> &() { &() }
/// #     fn graphic_after(&self) -> &() { &() }
/// # }
/// struct UniformChildren {
///     a: suzy::widget::Widget<NoChildren>,
///     b: suzy::widget::Widget<NoChildren>,
///     c: suzy::widget::Widget<NoChildren>,
///     d: suzy::widget::Widget<NoChildren>,
/// }
///
/// impl suzy::widget::WidgetData for UniformChildren {
///     fn init(init: &mut suzy::widget::WidgetInit<Self>) {}
///     suzy::children!(a, b, c, d; NoChildren);
///     type Graphic = ();
///     type GraphicAfter = ();
///     fn graphic(&self) -> &() { &() }
///     fn graphic_after(&self) -> &() { &() }
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
        {   $crate::widget::children::WidgetChildren::One(
                $crate::widget::NewWidget::as_widget(&self.$a),
            )
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildrenMut::One(
                $crate::widget::NewWidget::as_widget_mut(&mut self.$a),
            )
        }
    };
    ($a:ident => $at:ty, $b:ident => $bt:ty,) => {
        type ChildA = $at;
        type ChildB = $bt;
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildren::Two(
                $crate::widget::NewWidget::as_widget(&self.$a),
                $crate::widget::NewWidget::as_widget(&self.$b),
            )
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildrenMut::Two(
                $crate::widget::NewWidget::as_widget_mut(&mut self.$a),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$b),
            )
        }
    };
    ($a:ident => $at:ty, $b:ident => $bt:ty, $c:ident => $ct:ty,) => {
        type ChildA = $at;
        type ChildB = $bt;
        type ChildC = $ct;
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildren::Three(
                $crate::widget::NewWidget::as_widget(&self.$a),
                $crate::widget::NewWidget::as_widget(&self.$b),
                $crate::widget::NewWidget::as_widget(&self.$c),
            )
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildrenMut::Three(
                $crate::widget::NewWidget::as_widget_mut(&mut self.$a),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$b),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$c),
            )
        }
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
        {   $crate::widget::children::WidgetChildren::Four(
                $crate::widget::NewWidget::as_widget(&self.$a),
                $crate::widget::NewWidget::as_widget(&self.$b),
                $crate::widget::NewWidget::as_widget(&self.$c),
                $crate::widget::NewWidget::as_widget(&self.$d),
            )
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   $crate::widget::children::WidgetChildrenMut::Four(
                $crate::widget::NewWidget::as_widget_mut(&mut self.$a),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$b),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$c),
                $crate::widget::NewWidget::as_widget_mut(&mut self.$d),
            )
        }
    };
    ($a:ident, $b:ident ; $t:ty) => {
        $crate::children!($a => $t, $b => $t,);
    };
    ($a:ident, $b:ident, $c:ident ; $t:ty) => {
        $crate::children!($a => $t, $b => $t, $c => $t,);
    };
    ($a:ident, $b:ident, $c:ident, $d:ident ; $t:ty) => {
        $crate::children!($a => $t, $b => $t, $c => $t, $d => $t,);
    };
    ( $($x:ident),* ; $t:ty ) => {
        type ChildA = $t;
        type ChildB = ();
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec![
                $( $crate::widget::NewWidget::as_widget(&self.$x),)*
            ];
            $crate::widget::children::WidgetChildren::Uniform(list)
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec![
                $( $crate::widget::NewWidget::as_widget_mut(&mut self.$x),)*
            ];
            $crate::widget::children::WidgetChildrenMut::Uniform(list)
        }
    };
    ( $($x:ident => $xt:ty,)* ) => {
        type ChildA = ();
        type ChildB = ();
        type ChildC = ();
        type ChildD = ();
        fn children(&self)
            -> $crate::widget::children::WidgetChildren<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec![
                $( $crate::widget::NewWidget::as_widget(&self.$x).proxy(),)*
            ];
            $crate::widget::children::WidgetChildren::Varied(list)
        }

        fn children_mut(&mut self)
            -> $crate::widget::children::WidgetChildrenMut<
                Self::ChildA,Self::ChildB,Self::ChildC,Self::ChildD>
        {   let list = vec![
                $( $crate::widget::NewWidget::as_widget_mut(&mut self.$x)
                    .proxy_mut(),
                )*
            ];
            $crate::widget::children::WidgetChildrenMut::Varied(list)
        }
    };
}


#[cfg(test)]
mod tests {
    struct NoChildren {
    }
    impl super::widget::WidgetData for NoChildren {
        fn init(_init: &mut super::widget::WidgetInit<Self>) {}
        super::children!();
        type Graphic = ();
        type GraphicAfter = ();
        fn graphic(&self) -> &() { &() }
        fn graphic_after(&self) -> &() { &() }
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
        fn init(_init: &mut super::widget::WidgetInit<Self>) {}
        super::children!(a,b,c,d,e,f; NoChildren);
        type Graphic = ();
        type GraphicAfter = ();
        fn graphic(&self) -> &() { &() }
        fn graphic_after(&self) -> &() { &() }
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
