use yew::prelude::*;

use crate::{ui_components::collapsible_section::CollapsibleSection, yew_utils::raw_html};

/// Блок, с которого начинается калькулятор
#[function_component]
pub fn Intro() -> Html {
    html! {
        <>
            <h1>{"Калькулятор оптимальной смеси удобрений №2"}<br/>{" по методу Миттлайдера"}</h1>
            <CollapsibleSection description="Что это такое такое?">
                {raw_html(
                    r##"
                    <div>
                    <p>
                        Если вы хотите заниматься овощеводством
                        по <a href="/blog/tag/mittleider-metod/">методу Миттлайдера</a>,
                        но столкнулись с трудностями в
                        <a href="https://legom.info/blog/2010/3/18/70/">
                        составлении смеси удобрений
                        </a> &mdash; этот калькулятор вам поможет!
                    </p>
                    <p>
                        С помощью этого калькулятора можно составить смесь №2 или №2а из имеющихся
                        у вас комплексных и моно- удобрений так, чтобы выполнялись
                        <a href="/info/arts/element-conditions/">главные условия</a>
                        соотношения макроэлементов
                        (N&mdash;P<sub>2</sub>O<sub>5</sub>&mdash;K<sub>2</sub>O и
                        MgO&mdash;P<sub>2</sub>O<sub>5</sub>),
                        используя <strong>минимум удобрений</strong>.
                        Предпосадочная смесь №1 не требует сложных расчётов,
                        и её вы можете составить следуя
                        <a href="https://legom.info/blog/2010/3/18/70/#smes1">этой инструкции</a>.
                    </p>
                    <p class="warning">
                        <span class="warning">Внимание!</span> Продающиеся удобрения могут
                        отличаться по составу от указанных в таблице, хотя названия будут те же самые.
                        <span>Внимательно читайте состав удобрения на упаковке. Если он отличается,
                        <a href="#new-fert">добавьте его как новое удобрение</a>. <br/>
                        <a class="help" href="/info/arts/fertilizers-components/">
                            Как определить состав удобрения &rarr;
                        </a></span>
                    </p>
                    </div>
                    "##)}
            </CollapsibleSection>
        </>
    }
}
