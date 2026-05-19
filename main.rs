// =====================================================================
// BAKOME AI-BOT UNIFIED v1.0 – CORRIGÉ
// Auteur: Kitoko Bakome Fabrice Bandia (BAKOME)
// Rust – Envoi email + IA + Religion + Subventions + Dashboard + Conviction
// =====================================================================

use axum::{
    Router, routing::{get, post}, Json, extract::State, response::IntoResponse,
};
use lettre::{
    Message, SmtpTransport, Transport, message::Mailbox,
    transport::smtp::authentication::Credentials,
};
use redis::{Client as RedisClient, AsyncCommands};
use serde::{Serialize, Deserialize};
use std::{sync::Arc, collections::HashMap, env};
use tokio::sync::Mutex;
use anyhow::Result;
use dotenv::dotenv;
use chrono::Utc;
use uuid::Uuid;
use reqwest::Client as HttpClient;
use serde_json::json;
use tracing::{info, error, warn};
use tracing_subscriber;
use axum::response::Html;
use rand::Rng;

// ========================== STRUCTURES ==========================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
    pub received_at: i64,
    pub status: String,
    pub is_spam: bool,
    pub is_phishing: bool,
    pub ai_classification: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subvention {
    pub source: String,
    pub titre: String,
    pub url: String,
    pub montant: String,
    pub deadline: String,
}

#[derive(Clone)]
pub struct AppState {
    pub redis_client: RedisClient,
    pub smtp_relay: String,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub http_client: HttpClient,
    pub webhooks: Arc<Mutex<HashMap<String, WebhookConfig>>>,
}

// ========================== ENVOI D'EMAIL ==========================
pub async fn send_email(to: &str, subject: &str, body: &str, html: Option<&str>) -> Result<()> {
    let smtp_relay = env::var("SMTP_RELAY").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_user = env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = env::var("SMTP_PASS").unwrap_or_default();
    let from_email = env::var("FROM_EMAIL").unwrap_or_else(|_| "bot@localhost".to_string());

    let from: Mailbox = from_email.parse()?;
    let to: Mailbox = to.parse()?;

    let builder = Message::builder().from(from).to(to).subject(subject);
    let email = if let Some(html_content) = html {
        builder
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .body(String::from(body), lettre::message::header::ContentType::TEXT_PLAIN),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .body(
                                String::from(html_content),
                                lettre::message::header::ContentType::TEXT_HTML,
                            ),
                    ),
            )?
            .body(())
    } else {
        builder.body(String::from(body))?
    };

    let creds = Credentials::new(smtp_user, smtp_pass);
    let mailer = SmtpTransport::relay(&smtp_relay)?.credentials(creds).build();
    match mailer.send(&email) {
        Ok(_) => {
            info!("Email sent to {}", to);
            Ok(())
        }
        Err(e) => {
            error!("Send failed: {}", e);
            Err(anyhow::anyhow!("Send failed: {}", e))
        }
    }
}

// ========================== IA, PSYCHOLOGIE, RELIGION ==========================
async fn analyser_humeur(texte: &str) -> String {
    let positif = vec!["merci", "super", "heureux", "gentil", "love", "good", "bravo"];
    let negatif = vec!["triste", "souffre", "mal", "solitaire", "fatigué", "suis seul"];
    if positif.iter().any(|&m| texte.to_lowercase().contains(m)) {
        "positive".into()
    } else if negatif.iter().any(|&m| texte.to_lowercase().contains(m)) {
        "negative".into()
    } else {
        "neutre".into()
    }
}

async fn generer_blague() -> String {
    let blagues = vec![
        "Pourquoi les développeurs détestent la nature ? Parce qu'il y a trop de bugs 🐛".into(),
        "Pourquoi les programmeurs confondent toujours Noël et Halloween ? Parce que Oct 31 = Dec 25 🎄".into(),
        "Un testeur entre dans un bar. Il commande une bière. Il commande 0 bière. Il commande 999999999 bières. Il commande un canard. La serveuse dit : 'Mais vous êtes fou ?' Le testeur répond : 'Je vérifie les cas limites.'".into(),
    ];
    let mut rng = rand::thread_rng();
    blagues[rng.gen_range(0..blagues.len())].clone()
}

const BIBLE_VERSETS: &[(&str, &str)] = &[
    ("amour", "1 Corinthiens 13:7 : L'amour supporte tout, croit tout, espère tout, endure tout."),
    ("partage", "Actes 20:35 : Il y a plus de bonheur à donner qu'à recevoir."),
    ("espoir", "Jérémie 29:11 : Je connais les projets que j'ai pour vous, des projets de paix."),
];

const CORAN_VERSETS: &[(&str, &str)] = &[
    ("compassion", "Sourate 21:107 : Nous ne t'avons envoyé que par miséricorde pour l'univers."),
    ("generosite", "Sourate 2:261 : Ceux qui dépensent leurs biens dans le chemin d'Allah ressemblent à un grain qui produit sept épis."),
];

async fn citation_religieuse(sujet: &str, religion: &str) -> String {
    match religion {
        "chrétien" => {
            for (theme, texte) in BIBLE_VERSETS {
                if sujet.to_lowercase().contains(theme) {
                    return format!("📖 {} – {}", texte, theme);
                }
            }
            "📖 Psaume 37:25 : Je n'ai jamais vu le juste abandonné.".into()
        }
        "musulman" => {
            for (theme, texte) in CORAN_VERSETS {
                if sujet.to_lowercase().contains(theme) {
                    return format!("🕊️ {} – {}", texte, theme);
                }
            }
            "🕊️ Sourate 94:5 : Oui, avec la difficulté vient la facilité.".into()
        }
        _ => "Chaque tradition enseigne la bonté et le partage.".into(),
    }
}

async fn ia_multi_modele(prompt: &str, style: &str) -> String {
    let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
    let client = HttpClient::new();
    let prompt_full = format!(
        "Style: {}. Réponds avec empathie, parfois avec humour, parfois avec sagesse: {}",
        style, prompt
    );
    let response = client
        .post(format!("{}/api/generate", ollama_url))
        .json(&json!({
            "model": model,
            "prompt": prompt_full,
            "stream": false
        }))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await;
    match response {
        Ok(resp) => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(text) = json.get("response").and_then(|r| r.as_str()) {
                    return text.to_string();
                }
            }
        }
        Err(e) => {
            warn!("IA multi-modèle failed: {}", e);
        }
    }
    format!(
        "(Simulation IA) Je comprends ton message '{}' en style '{}'. Je vais t'aider.",
        &prompt[..prompt.len().min(50)],
        style
    )
}

async fn generer_reponse_spirituelle(texte: &str, religion: &str, style: &str) -> String {
    let humeur = analyser_humeur(texte).await;
    let citation = citation_religieuse(texte, religion).await;
    let base_ia = ia_multi_modele(texte, style).await;
    let mut reponse = format!("😊 Humeur détectée: {}\n\n{}\n\n{}", humeur, citation, base_ia);
    if humeur == "negative" {
        reponse.push_str(&format!(
            "\n\n😂 Tiens, une petite blague pour te changer les idées : {}",
            generer_blague().await
        ));
    }
    reponse
}

// ========================== SCANNEUR SUBVENTIONS ==========================
async fn scanner_subventions() -> Vec<Subvention> {
    vec![
        Subvention {
            source: "Gitcoin".into(),
            titre: "Open Source Software Round".into(),
            url: "https://gitcoin.co/grants".into(),
            montant: "Variable + matching".into(),
            deadline: "2026-06-30".into(),
        },
        Subvention {
            source: "Drips Network".into(),
            titre: "Drips Ecosystem Funding".into(),
            url: "https://drips.network".into(),
            montant: "Streaming USDC".into(),
            deadline: "continu".into(),
        },
        Subvention {
            source: "IssueHunt".into(),
            titre: "Bounty Board".into(),
            url: "https://issuehunt.io".into(),
            montant: "Par tâche".into(),
            deadline: "continu".into(),
        },
        Subvention {
            source: "NGI0 Commons Fund".into(),
            titre: "NGI0 Commons Fund".into(),
            url: "https://nlnet.nl/commonsfund".into(),
            montant: "5k-50k €".into(),
            deadline: "2026-06-01".into(),
        },
        Subvention {
            source: "NSF PESOSE".into(),
            titre: "NSF PESOSE".into(),
            url: "https://nsf.gov".into(),
            montant: "Jusqu'à 300k $".into(),
            deadline: "2026-08-15".into(),
        },
        Subvention {
            source: "Stellar Drips Wave".into(),
            titre: "Stellar Drips Wave".into(),
            url: "https://stellar.org".into(),
            montant: "Streaming XLM/USDC".into(),
            deadline: "2026-07-01".into(),
        },
    ]
}

// ========================== API ROUTES ==========================
async fn api_chat(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let texte = payload.get("message").and_then(|v| v.as_str()).unwrap_or("");
    let religion = payload.get("religion").and_then(|v| v.as_str()).unwrap_or("neutre");
    let style = payload.get("style").and_then(|v| v.as_str()).unwrap_or("bienveillant");
    let reponse = generer_reponse_spirituelle(texte, religion, style).await;
    Json(json!({ "reponse": reponse }))
}

async fn api_subventions() -> impl IntoResponse {
    let subs = scanner_subventions().await;
    Json(json!({ "subventions": subs }))
}

async fn api_send_email(Json(req): Json<SendEmailRequest>) -> impl IntoResponse {
    match send_email(&req.to, &req.subject, &req.body, req.html.as_deref()).await {
        Ok(_) => Json(json!({ "status": "sent", "to": req.to })),
        Err(e) => Json(json!({ "status": "error", "message": e.to_string() })),
    }
}

async fn api_stats(State(state): State<AppState>) -> impl IntoResponse {
    let mut conn = state.redis_client.get_async_connection().await.unwrap();
    let queue_len: usize = conn.llen("email_queue").await.unwrap_or(0);
    Json(json!({
        "queue_length": queue_len,
        "webhooks_count": state.webhooks.lock().await.len(),
        "status": "healthy",
    }))
}

// ========================== DASHBOARD HTML ==========================
const DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>BAKOME AI-BOT</title><style>
*{margin:0;padding:0;box-sizing:border-box;font-family:monospace}
body{background:linear-gradient(135deg,#0a0a0a,#1a1a2e);min-height:100vh;padding:20px}
.container{max-width:1400px;margin:0 auto}
h1{background:linear-gradient(135deg,#0ff,#f0f);-webkit-background-clip:text;background-clip:text;color:transparent}
.card{background:rgba(0,0,0,0.6);border-radius:24px;padding:20px;margin-bottom:20px;border:1px solid #0ff}
button{background:#0ff;color:#000;padding:10px 20px;border:none;border-radius:30px;cursor:pointer}
input,textarea{width:100%;padding:10px;margin:10px 0;background:#1a1a2e;border:1px solid #333;color:white}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:20px}
@media(max-width:900px){.grid{grid-template-columns:1fr}}
</style>
</head>
<body>
<div class="container">
<h1>🤖 BAKOME AI-BOT UNIFIED v1.0</h1>
<p style="color:#888; margin-bottom:20px">Email + IA + Religion + Subventions + Humour + Conviction</p>
<div class="grid">
<div class="card"><h2>💬 Chat avec AI</h2>
<textarea id="chatMsg" rows="3" placeholder="Parle moi..."></textarea>
<select id="religion"><option>neutre</option><option>chrétien</option><option>musulman</option></select>
<select id="style"><option>bienveillant</option><option>humoriste</option><option>pasteur</option></select>
<button onclick="sendChat()">Envoyer</button>
<pre id="chatReponse" style="margin-top:20px; white-space:pre-wrap"></pre>
</div>
<div class="card"><h2>📡 Subventions</h2><button onclick="fetchSubventions()">Scanner</button><pre id="subs"></pre></div>
<div class="card"><h2>✉️ Envoyer Email</h2>
<input id="emailTo" placeholder="destinataire"><input id="emailSubject" placeholder="Sujet">
<textarea id="emailBody" rows="3" placeholder="Message"></textarea>
<button onclick="sendEmail()">Envoyer</button>
</div>
<div class="card"><h2>📊 Stats</h2><pre id="stats"></pre></div>
</div>
</div>
<script>
async function sendChat(){let msg=document.getElementById('chatMsg').value; if(!msg)return;
let rel=document.getElementById('religion').value; let style=document.getElementById('style').value;
let res=await fetch('/api/chat',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({message:msg,religion:rel,style:style})});
let data=await res.json(); document.getElementById('chatReponse').innerText=data.reponse;
}
async function fetchSubventions(){let res=await fetch('/api/subventions'); let data=await res.json(); document.getElementById('subs').innerText=JSON.stringify(data.subventions,null,2);}
async function sendEmail(){let to=document.getElementById('emailTo').value; let subject=document.getElementById('emailSubject').value; let body=document.getElementById('emailBody').value;
let res=await fetch('/api/send',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({to,subject,body})});
let data=await res.json(); alert(data.status);}
async function fetchStats(){let res=await fetch('/api/stats'); let data=await res.json(); document.getElementById('stats').innerText=JSON.stringify(data,null,2);}
setInterval(fetchStats,10000); fetchStats(); fetchSubventions();
</script>
</body></html>"#;

async fn dashboard() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

// ========================== PROCESSUS QUEUE ==========================
pub async fn process_queue(redis_client: RedisClient, state: AppState) -> Result<()> {
    let mut conn = redis_client.get_async_connection().await?;
    loop {
        let email_json: Option<String> = conn.rpop("email_queue").await?;
        if let Some(json_str) = email_json {
            match serde_json::from_str::<Email>(&json_str) {
                Ok(email) => info!("Processing email: {}", email.id),
                Err(e) => error!("Failed to parse email from queue: {}", e),
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

// ========================== MAIN ==========================
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let redis_client = RedisClient::open(redis_url.clone())?;

    let state = AppState {
        redis_client: redis_client.clone(),
        smtp_relay: env::var("SMTP_RELAY").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
        smtp_user: env::var("SMTP_USER").unwrap_or_default(),
        smtp_pass: env::var("SMTP_PASS").unwrap_or_default(),
        http_client: HttpClient::new(),
        webhooks: Arc::new(Mutex::new(HashMap::new())),
    };

    // Lancer le processeur de queue dans une tâche séparée
    let queue_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = process_queue(redis_client, queue_state).await {
            error!("Queue processor stopped: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/api/chat", post(api_chat))
        .route("/api/subventions", get(api_subventions))
        .route("/api/send", post(api_send_email))
        .route("/api/stats", get(api_stats))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    info!("🌐 BAKOME AI-BOT: http://localhost:3001");
    axum::serve(listener, app).await?;

    Ok(())
}
