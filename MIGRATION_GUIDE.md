# Django → Rust + Dioxus Migration Guide for RentoLink

## Executive Summary

This guide maps every Django concept in your RentoLink project to its Rust equivalent. The starter project in `rento-rs/` provides a working foundation you can build upon.

---

## 1. Architecture Comparison

```
┌─────────────────────────────────────────────────────────────────┐
│                        DJANGO (Current)                         │
├─────────────────────────────────────────────────────────────────┤
│  React/Vue Frontend  ←→  Django REST API  ←→  PostgreSQL      │
│                         ├─ DRF ViewSets                         │
│                         ├─ Django ORM                         │
│                         ├─ Djoser Auth                        │
│                         ├─ Celery Tasks                     │
│                         ├─ Django Admin                     │
│                         └─ django-filter                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     RUST + DIOXUS (Target)                      │
├─────────────────────────────────────────────────────────────────┤
│  Dioxus Frontend  ←→  Axum API  ←→  sqlx  ←→  PostgreSQL      │
│                      ├─ Tower Middleware                        │
│                      ├─ sqlx (compile-time SQL)                 │
│                      ├─ Custom JWT Auth                       │
│                      ├─ Tokio Async Tasks                     │
│                      ├─ Dioxus Admin Dashboard              │
│                      └─ Custom Query Builders                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Model-by-Model Translation

### AccountUser (Django → Rust)

**Django:**
```python
class AccountUser(AbstractBaseUser, PermissionsMixin):
    id = models.UUIDField(primary_key=True, default=uuid.uuid4, editable=False)
    email = models.EmailField(max_length=255, unique=True)
    username = models.CharField(max_length=255, unique=True)
    role = models.CharField(max_length=50, choices=Role.choices)
    # ... 30+ fields
```

**Rust (sqlx):**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUser {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub role: UserRole,  // Enum type in PostgreSQL
    // ... all fields
}
```

**Key differences:**
- No `objects` manager → Use `sqlx::query_as!()` directly
- No signals → Use database triggers or explicit function calls
- No `save()` override → Use explicit `update_user()` functions
- No proxy models (Agent, Client) → Use filtered queries or views

---

## 3. ViewSet → Axum Handler Mapping

### Django DRF ViewSet:
```python
class AgentViewSet(viewsets.ModelViewSet):
    queryset = Agent.objects.all()
    serializer_class = AgentCreateSerializer
    permission_classes = [IsAdminOrStaff]

    @action(detail=False, methods=['GET'], url_path='my-profile')
    def my_profile(self, request):
        agent = Agent.objects.get(id=request.user.id)
        return Response(AgentSerializer(agent).data)
```

### Rust Axum Handler:
```rust
pub async fn get_my_profile(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> Result<Json<serde_json::Value>> {
    let agent: AgentProfile = sqlx::query_as(
        "SELECT * FROM agent_profiles WHERE user_id = $1"
    )
    .bind(auth.user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(Json(serde_json::json!({ ... })))
}
```

**Route registration:**
```rust
.route("/agents/me", get(agents::get_my_profile))
```

---

## 4. Serializer → Rust DTO Mapping

### Django Serializer:
```python
class RegistrationSerializer(serializers.ModelSerializer):
    password = serializers.CharField(write_only=True, min_length=8)
    verification_code = serializers.CharField(write_only=True, required=True)

    def validate(self, attrs):
        # Custom validation logic
        pass

    def create(self, validated_data):
        # Create user logic
        pass
```

### Rust DTO:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    pub phone_number: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub verification_code: String,
}

// Validation happens automatically via extractor
pub async fn register(
    Json(req): Json<RegisterRequest>,  // Validates on deserialization
) -> Result<Json<serde_json::Value>> {
    req.validate()?;  // Explicit validation
    // ... create logic
}
```

---

## 5. Permission System Migration

### Django Permissions:
```python
class IsAdminOrAgent(permissions.BasePermission):
    def has_permission(self, request, view):
        return request.user.is_authenticated and (
            request.user.is_superuser or 
            request.user.role == 'AGENT'
        )
```

### Rust Extractors:
```rust
pub struct RequireAgentOrAdmin(pub AuthenticatedUser);

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for RequireAgentOrAdmin {
    type Rejection = RentoError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts.extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| RentoError::Auth("Authentication required".to_string()))?;

        if !auth_user.is_admin() && !auth_user.is_agent() {
            return Err(RentoError::Authorization("Agent or admin access required".to_string()));
        }

        Ok(Self(auth_user))
    }
}
```

**Usage in handler:**
```rust
pub async fn some_handler(
    _auth: RequireAgentOrAdmin,  // Fails automatically if not authorized
) -> Result<Json<...>> { ... }
```

---

## 6. Authentication Migration (JWT Cookies)

### Django (djoser + simplejwt):
```python
class CustomTokenObtainPairView(TokenObtainPairView):
    def post(self, request, *args, **kwargs):
        response = super().post(request, *args, **kwargs)
        response.set_cookie(
            settings.SIMPLE_JWT['AUTH_COOKIE'],
            response.data['access'],
            httponly=True,
            secure=True,
            samesite='Lax',
        )
        return response
```

### Rust (Axum + custom JWT):
```rust
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Response> {
    // ... validate user ...

    let (access_token, refresh_token) = state.auth.generate_tokens(...)?;

    let mut response = Json(LoginResponse { ... }).into_response();

    // Set cookies
    let access_cookie = Cookie::build(("access_token", access_token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::minutes(60));

    response.headers_mut().append(
        SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}
```

---

## 7. Email Service Migration

### Django:
```python
from django.core.mail import EmailMultiAlternatives
email = EmailMultiAlternatives(subject, text, from_email, [to])
email.attach_alternative(html, "text/html")
email.send()
```

### Rust (lettre):
```rust
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, MultiPart, SinglePart},
};

let email = Message::builder()
    .from("RentoLink <noreply@rentolink.com>".parse()?)
    .to(to_email.parse()?)
    .subject(subject)
    .multipart(
        MultiPart::alternative()
            .singlepart(SinglePart::plain(text_content))
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_content)
            )
    )?;

let mailer = SmtpTransport::relay("smtp.example.com")?
    .credentials(Credentials::new("user".to_string(), "pass".to_string()))
    .build();

mailer.send(&email)?;
```

---

## 8. Celery Tasks → Tokio Tasks

### Django Celery:
```python
@shared_task
def cleanup_expired_verifications():
    expired_count = PhoneVerification.objects.filter(
        expires_at__lt=timezone.now()
    ).delete()[0]
    return f"Cleaned up {expired_count} expired verifications"
```

### Rust (Tokio + apalis):
```rust
use apalis::prelude::*;
use chrono::Utc;

#[derive(Default, Debug, Clone)]
pub struct CleanupJob;

impl Job for CleanupJob {
    const NAME: &'static str = "cleanup_expired_verifications";
}

pub async fn cleanup_expired_verifications(
    _job: CleanupJob,
    pool: Data<PgPool>,
) -> Result<String, Error> {
    let result = sqlx::query(
        "DELETE FROM phone_verifications WHERE expires_at < $1"
    )
    .bind(Utc::now())
    .execute(&pool)
    .await?;

    Ok(format!("Cleaned up {} expired verifications", result.rows_affected()))
}

// Schedule the job
let worker = WorkerBuilder::new("cleanup-worker")
    .with_storage(storage.clone())
    .build_fn(cleanup_expired_verifications);

// Run every hour
let schedule = Schedule::from_str("0 * * * * *")?;
Monitor::new()
    .register_with_schedule(schedule, worker)
    .run()
    .await?;
```

---

## 9. Django Admin → Dioxus Admin

### Django Admin:
```python
@admin.register(AgentProfile)
class AgentProfileAdmin(ModelAdmin, GuardedModelAdmin):
    list_display = ['user', 'agent_id', 'total_commissions', ...]
    readonly_fields = ('agent_id',)
```

### Dioxus Admin Dashboard:
```rust
#[component]
fn AdminDashboard() -> Element {
    let agents = use_resource(|| async {
        api::get_agents().await
    });

    rsx! {
        div { class: "admin-dashboard",
            h1 { "Admin Dashboard" }

            table { class: "data-table",
                thead {
                    tr {
                        th { "User" }
                        th { "Agent ID" }
                        th { "Total Commissions" }
                        th { "Actions" }
                    }
                }
                tbody {
                    match agents.read().as_ref() {
                        Some(Ok(data)) => rsx! {
                            for agent in data {
                                tr {
                                    td { "{agent.user_name}" }
                                    td { "{agent.agent_id}" }
                                    td { "KSh {agent.total_commissions}" }
                                    td {
                                        button { "Edit" }
                                        button { "Delete" }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! { tr { td { colspan: "4", "Error: {e}" } } },
                        None => rsx! { tr { td { colspan: "4", "Loading..." } } },
                    }
                }
            }
        }
    }
}
```

---

## 10. URL Routing Comparison

### Django URLs:
```python
path('auth/', include('accounts.urls')),
path('api/', include('listings.urls')),
path('api/', include('subscriptions.urls')),
```

### Rust Router:
```rust
let app = Router::new()
    .route("/auth/register", post(auth::register))
    .route("/auth/login", post(auth::login))
    .route("/properties", get(properties::list))
    .route("/properties", post(properties::create))
    .route("/properties/:id", get(properties::get))
    .route("/properties/:id", patch(properties::update))
    .route("/properties/:id", delete(properties::delete))
    .route("/subscriptions/plans", get(subscriptions::list_plans))
    // ... etc
```

---

## 11. File Upload Handling

### Django (DRF):
```python
class PropertyInformationSerializer(serializers.ModelSerializer):
    property_images_files = serializers.ListField(
        child=serializers.ImageField(),
        write_only=True,
    )
```

### Rust (Axum Multipart):
```rust
use axum::extract::Multipart;

pub async fn upload_images(
    State(state): State<Arc<AppState>>,
    Path(property_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>> {
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        let data = field.bytes().await?;

        if name == "property_images_files" {
            // Save file
            let filename = format!("{}.jpg", Uuid::new_v4());
            let path = format!("uploads/{}", filename);
            tokio::fs::write(&path, &data).await?;

            // Save to database
            sqlx::query("INSERT INTO property_images ...")
                .bind(property_id)
                .bind(&path)
                .execute(&state.db.pool)
                .await?;
        }
    }

    Ok(Json(serde_json::json!({"detail": "Images uploaded"})))
}
```

---

## 12. Testing Strategy

### Django Test:
```python
class TestRegistration(APITestCase):
    def test_register_user(self):
        response = self.client.post('/auth/register/', {
            'email': 'test@example.com',
            'password': 'password123',
            'verification_code': '123456',
        })
        self.assertEqual(response.status_code, 201)
```

### Rust Test:
```rust
#[tokio::test]
async fn test_register_user() {
    let state = create_test_state().await;
    let app = create_test_app(state);

    let response = app
        .oneshot(Request::builder()
            .method("POST")
            .uri("/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(json!({
                "email": "test@example.com",
                "password": "password123",
                "verification_code": "123456",
            }).to_string()))
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
```

---

## 13. Deployment

### Django (Gunicorn + Nginx):
```bash
gunicorn rento.wsgi:application --bind 0.0.0.0:8000
```

### Rust (Binary + Nginx/Reverse Proxy):
```bash
# Build release binary
cargo build --release -p rento-api

# Run directly
./target/release/rento-api

# Or with systemd
systemctl start rento-api
```

**Docker:**
```bash
docker-compose up --build
```

---

## 14. Performance Expectations

| Metric | Django | Rust + Axum | Improvement |
|--------|--------|-------------|-------------|
| Request latency | 50-100ms | 5-15ms | **5-10x faster** |
| Memory usage | 200-500MB | 50-100MB | **4-5x lower** |
| CPU usage | High | Very Low | **~10x lower** |
| Cold start | 5-10s | <1s | **~10x faster** |
| Concurrent requests | 100-500 | 10,000+ | **20x+ more** |

---

## 15. Migration Checklist

### Phase 1: Foundation (Week 1-2)
- [ ] Set up PostgreSQL with new schema
- [ ] Implement core models and auth system
- [ ] Create basic CRUD handlers for users
- [ ] Set up JWT authentication with cookies
- [ ] Create Dioxus login/register pages

### Phase 2: Core Features (Week 3-4)
- [ ] Implement property CRUD
- [ ] Implement property unit CRUD
- [ ] Implement image upload
- [ ] Create property listing pages
- [ ] Implement search and filtering

### Phase 3: Business Logic (Week 5-6)
- [ ] Implement subscription system
- [ ] Implement commission tracking
- [ ] Implement agent registration flow
- [ ] Implement role conversion
- [ ] Create admin dashboard

### Phase 4: Integrations (Week 7-8)
- [ ] Email service (lettre)
- [ ] SMS service (Infobip)
- [ ] WhatsApp OTP
- [ ] OAuth (Google, GitHub, Facebook)
- [ ] Payment integration (M-Pesa/Stripe)

### Phase 5: Polish (Week 9-10)
- [ ] Background tasks (apalis)
- [ ] Caching (Redis)
- [ ] Rate limiting
- [ ] Logging and monitoring
- [ ] Testing and documentation

---

## Next Steps

1. **Review the starter project** in `rento-rs/`
2. **Set up PostgreSQL** and run the migrations
3. **Implement the auth handlers** first (most critical)
4. **Test with the Dioxus frontend** as you build
5. **Iterate feature by feature**, not file by file

The starter project provides the complete structure. Focus on implementing one handler at a time, testing it with `curl` or the frontend, then move to the next.

Good luck with the migration! 🦀
