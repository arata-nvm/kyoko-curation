use std::str::FromStr;

use rand::prelude::SliceRandom;
use reqwasm::http::Request;
use rss::Channel;
use scraper::{Html, Selector};
use yew::prelude::*;

const DEFAULT_THUMB: &str = "https://via.placeholder.com/120x90";
const KYOKO_FEED: &str =
    "https://murmuring-cove-94903.herokuapp.com/https://kyoko-np.net/index.xml";
const FEEDS: [&str; 2] = [
    "https://murmuring-cove-94903.herokuapp.com/https://www.nhk.or.jp/rss/news/cat0.xml",
    "https://murmuring-cove-94903.herokuapp.com/https://www.nhk.or.jp/rss/news/cat3.xml",
];

#[derive(Clone, Debug, PartialEq)]
struct Article {
    pub title: String,
    pub description: String,
    pub link: String,
    pub thumb_link: Option<String>,
}

async fn fetch_all_articles() -> Vec<Article> {
    let kyoko_articles = fetch_articles(KYOKO_FEED).await;
    let mut articles = Vec::new();
    for feed in &FEEDS {
        articles.extend(fetch_articles(feed).await);
    }

    let mut rng = rand::thread_rng();
    let mut articles: Vec<Article> = articles.choose_multiple(&mut rng, 23).cloned().collect();
    articles.extend(kyoko_articles.choose_multiple(&mut rng, 2).cloned());
    articles.shuffle(&mut rng);
    articles
}

async fn fetch_articles(feed: &str) -> Vec<Article> {
    let content = Request::get(feed)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let channel = Channel::from_str(&content).unwrap();
    channel
        .items()
        .iter()
        .map(|item| Article {
            title: item.title().unwrap().to_string(),
            description: item.description().unwrap().to_string(),
            link: item.link().unwrap().to_string(),
            thumb_link: None,
        })
        .collect()
}

async fn fetch_article_thumb(article: &mut Article) {
    let html = Request::get(&format!(
        "https://murmuring-cove-94903.herokuapp.com/{}",
        article.link
    ))
    .send()
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    let fragment = Html::parse_fragment(&html);
    let selector = Selector::parse("meta[property='og:image']").unwrap();
    article.thumb_link = fragment
        .select(&selector)
        .next()
        .and_then(|elem| elem.value().attr("content"))
        .map(|link| link.to_string());
}

#[derive(Clone, Properties, PartialEq)]
struct ArticleCardProps {
    article: Article,
}

#[function_component(ArticleCard)]
fn article_card(ArticleCardProps { article }: &ArticleCardProps) -> Html {
    let thumb_link = article
        .thumb_link
        .clone()
        .unwrap_or_else(|| DEFAULT_THUMB.to_string());

    html! {
        <div class="article-card">
            <img class="card-thumbnail" src={thumb_link} />
            <div class="card-meta">
                <h3 class="card-title"><a href={ article.link.clone() } target="_blank" rel="noopener">{ article.title.clone() }</a></h3>
                <p class="card-description">{ article.description.clone() }</p>
            </div>
        </div>
    }
}

#[derive(Clone, Properties, PartialEq)]
struct ArticleListProps {
    articles: Vec<Article>,
}

#[function_component(ArticleList)]
fn article_list(ArticleListProps { articles }: &ArticleListProps) -> Html {
    match articles.len() {
        0 => html! {
            <p>{ "loading..." }</p>
        },
        _ => articles
            .iter()
            .map(|article| {
                html! {
                    <ArticleCard article={article.clone()} />
                }
            })
            .collect(),
    }
}

#[function_component(App)]
fn app() -> Html {
    let articles = use_state(Vec::new);
    {
        let articles = articles.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut fetched_articles = fetch_all_articles().await;
                    articles.set(fetched_articles.clone());
                    for i in 0..fetched_articles.len() {
                        fetch_article_thumb(&mut fetched_articles[i]).await;
                        articles.set(fetched_articles.clone());
                    }
                });
                || ()
            },
            (),
        );
    }

    html! {
        <>
            <div class="header">
                <h1 class="header-title">{ "虚構キュレーター" }</h1>
                <p class="header-quote">{ "うそはうそであると見抜ける人でないと(掲示板を使うのは)難しい" }<i>{ " - 西村博之" }</i></p>
            </div>
            <div class="article-list">
                <ArticleList articles={(*articles).clone()} />
            </div>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
