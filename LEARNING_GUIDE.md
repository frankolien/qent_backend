# Riverpod State Management in Qent -- A Codebase-Driven Learning Guide

This guide teaches Riverpod state management, API integration, and user preferences
using real code from the Qent car-rental mobile app. Every example references actual
files and line numbers so you can follow along in your editor.

---

## Table of Contents

1. [What is Riverpod?](#1-what-is-riverpod)
2. [Core Concepts](#2-core-concepts)
3. [How Qent Uses Riverpod](#3-how-qent-uses-riverpod)
4. [Auth Flow Deep Dive](#4-auth-flow-deep-dive)
5. [API Integration Pattern](#5-api-integration-pattern)
6. [State Management Patterns](#6-state-management-patterns)
7. [User Preferences with SharedPreferences](#7-user-preferences-with-sharedpreferences)
8. [Real-time with WebSocket + StreamProvider](#8-real-time-with-websocket--streamprovider)
9. [Common Patterns & Anti-patterns](#9-common-patterns--anti-patterns)
10. [Exercises](#10-exercises)

---

## 1. What is Riverpod?

Riverpod is a reactive state management framework for Flutter. It is the successor
to the original `Provider` package, built by the same author (Remi Rousselet), but
designed from scratch to fix fundamental limitations.

### Why Riverpod over Provider or Bloc?

| Feature                    | Provider       | Bloc           | Riverpod       |
|----------------------------|----------------|----------------|----------------|
| Compile-safe               | No (runtime)   | Partial        | Yes            |
| Independent of widget tree | No             | No             | Yes            |
| Multiple providers of same type | Hard      | Hard           | Easy           |
| Auto-dispose               | Manual         | Manual         | Built-in       |
| Testing                    | Needs context  | Needs context  | No context     |
| Async support              | Limited        | Streams only   | First-class    |

Qent uses Riverpod because:
- Providers are **global singletons** declared as top-level variables -- no widget tree coupling
- `FutureProvider` maps naturally to REST API calls
- `Notifier` gives fine-grained control for auth state machines
- `ref.invalidate()` makes cache-busting trivial after mutations

---

## 2. Core Concepts

### Provider Types at a Glance

```
+-------------------------------------------------------------------+
|                     RIVERPOD PROVIDER TYPES                       |
+-------------------------------------------------------------------+
|                                                                   |
|  Provider<T>            -- Synchronous, read-only value           |
|  StateProvider<T>       -- Simple mutable value (counter, toggle) |
|  FutureProvider<T>      -- One-shot async data (API fetch)        |
|  StreamProvider<T>      -- Continuous async stream                |
|  NotifierProvider<N,T>  -- Complex state + methods (class-based)  |
|  AsyncNotifierProvider  -- Like Notifier but state is AsyncValue  |
|                                                                   |
+-------------------------------------------------------------------+
```

### Data Flow

```
  +------------------+     ref.watch()      +------------------+
  |                  | <------------------- |                  |
  |    Provider      |                      |     Widget       |
  |  (holds state)   | ------------------> |  (ConsumerWidget) |
  |                  |     rebuilds UI      |                  |
  +------------------+                      +------------------+
         |                                         |
         | ref.read()                              | ref.read()
         | (actions, one-off reads)                | (button taps)
         v                                         v
  +------------------+                      +------------------+
  |   Other Provider  |                      |  Side Effects    |
  |   (dependency)    |                      |  (navigate, etc) |
  +------------------+                      +------------------+
```

### The `ref` Object

Every provider and every `ConsumerWidget` gets a `ref`. This is how you interact
with other providers:

- `ref.watch(provider)` -- Subscribe to changes. Widget rebuilds when value changes.
- `ref.read(provider)` -- Read current value once. No subscription. Use in callbacks.
- `ref.listen(provider, callback)` -- Listen for changes, run side effects.
- `ref.invalidate(provider)` -- Force the provider to re-compute next time it is read.

### ConsumerWidget vs ConsumerStatefulWidget

```dart
// Stateless -- use when you just need to watch providers
class MyPage extends ConsumerWidget {
  Widget build(BuildContext context, WidgetRef ref) {
    final data = ref.watch(someProvider);
    // ...
  }
}

// Stateful -- use when you also need initState, controllers, etc.
class MyPage extends ConsumerStatefulWidget {
  ConsumerState<MyPage> createState() => _MyPageState();
}
class _MyPageState extends ConsumerState<MyPage> {
  // Access ref via `ref` (no WidgetRef parameter needed)
  Widget build(BuildContext context) {
    final data = ref.watch(someProvider);
    // ...
  }
}
```

---

## 3. How Qent Uses Riverpod

### Provider Architecture Map

```
+========================================================================+
|                        QENT PROVIDER ARCHITECTURE                      |
+========================================================================+
|                                                                        |
|  FOUNDATION LAYER (Singletons)                                         |
|  +---------------------+    +---------------------+                    |
|  | apiClientProvider    |    | wsServiceProvider    |                    |
|  | Provider<ApiClient>  |    | Provider<WsService>  |                    |
|  +----------+----------+    +----------+----------+                    |
|             |                          |                               |
|  DATA SOURCE LAYER                     |                               |
|  +----------v----------+    +---------v-----------+                    |
|  | apiAuthDataSource    |    | apiChatDataSource    |                    |
|  | Provider             |    | Provider             |                    |
|  +----------+----------+    +---------+-----------+                    |
|             |                         |                                |
|  +----------v----------+    +--------v------------+                    |
|  | apiCarDataSource     |    | storiesProvider      |                    |
|  | Provider             |    | FutureProvider       |                    |
|  +----------+----------+    +---------------------+                    |
|             |                                                          |
|  CONTROLLER / STATE LAYER                                              |
|  +----------v--------------+   +---------------------+                 |
|  | authControllerProvider   |   | searchController    |                 |
|  | NotifierProvider         |   | NotifierProvider    |                 |
|  | (AuthController,         |   | (SearchController,  |                 |
|  |  AuthState)              |   |  SearchState)       |                 |
|  +----------+---+----------+   +----------+----------+                 |
|             |   |                         |                            |
|  DATA LAYER |   | invalidates on logout   |                            |
|  +----------v---v----------+   +----------v----------+                 |
|  | carsProvider            |   | filteredCarsProvider |                 |
|  | FutureProvider<List>    |   | FutureProvider<List> |                 |
|  +-------------------------+   +---------------------+                 |
|  | favoriteCarsProvider    |   | hostStatsProvider    |                 |
|  | FutureProvider<List>    |   | FutureProvider       |                 |
|  +-------------------------+   +---------------------+                 |
|  | favoriteIdsProvider     |   | hostPendingBookings  |                 |
|  | NotifierProvider<Set>   |   | FutureProvider<List> |                 |
|  +-------------------------+   +---------------------+                 |
|                                                                        |
|  CHAT / MESSAGING LAYER                                                |
|  +-------------------------+   +---------------------+                 |
|  | chatsProvider           |   | messagesProvider     |                 |
|  | FutureProvider<List>    |   | FutureProvider.family|                 |
|  +-------------------------+   +---------------------+                 |
|  | chatsStreamProvider     |   | messagesStreamProvider|                |
|  | StreamProvider<List>    |   | StreamProvider.family |                |
|  +-------------------------+   +---------------------+                 |
|  | chatControllerProvider  |   | onlineStatusStream   |                |
|  | Provider<ChatController>|   | StreamProvider.family |                |
|  +-------------------------+   +---------------------+                 |
|                                                                        |
|  NOTIFICATION LAYER                                                    |
|  +-------------------------------+                                     |
|  | currentUserNotificationsProvider |                                   |
|  | StreamProvider<List>            |  (watches authControllerProvider)  |
|  +-------------------------------+                                     |
|                                                                        |
+========================================================================+
```

### File Locations

| Provider                       | File                                                                        |
|--------------------------------|-----------------------------------------------------------------------------|
| `apiClientProvider`            | `mobile/lib/features/auth/presentation/providers/auth_providers.dart:9`      |
| `apiAuthDataSourceProvider`    | `mobile/lib/features/auth/presentation/providers/auth_providers.dart:12`     |
| `apiCarDataSourceProvider`     | `mobile/lib/features/auth/presentation/providers/auth_providers.dart:18`     |
| `authControllerProvider`       | `mobile/lib/features/auth/presentation/providers/auth_providers.dart:24`     |
| `carsProvider`                 | `mobile/lib/features/home/presentation/providers/car_providers.dart:8`       |
| `carProvider` (single)         | `mobile/lib/features/home/presentation/providers/car_providers.dart:20`      |
| `favoriteIdsProvider`          | `mobile/lib/features/home/presentation/providers/car_providers.dart:79`      |
| `searchControllerProvider`     | `mobile/lib/features/search/presentation/providers/search_providers.dart:11` |
| `filteredCarsProvider`         | `mobile/lib/features/search/presentation/providers/search_providers.dart:21` |
| `hostStatsProvider`            | `mobile/lib/features/dashboard/presentation/providers/dashboard_providers.dart:80` |
| `hostPendingBookingsProvider`  | `mobile/lib/features/dashboard/presentation/providers/dashboard_providers.dart:130`|
| `chatsProvider`                | `mobile/lib/features/chat/presentation/controllers/chat_controller.dart:16`  |
| `messagesProvider`             | `mobile/lib/features/chat/presentation/controllers/chat_controller.dart:34`  |
| `chatControllerProvider`       | `mobile/lib/features/chat/presentation/controllers/chat_controller.dart:110` |
| `wsServiceProvider`            | `mobile/lib/core/services/websocket_service.dart:187`                        |
| `storiesProvider`              | `mobile/lib/features/chat/presentation/providers/stories_providers.dart:58`  |
| `currentUserNotificationsProvider` | `mobile/lib/features/notifications/presentation/providers/notification_providers.dart:28` |

---

## 4. Auth Flow Deep Dive

### The Complete Sign-In Lifecycle

```
  APP LAUNCH
      |
      v
  main() in main.dart (line 20)
      |
      |--> ApiClient().initialize()          // Loads token from SharedPreferences
      |--> runApp(ProviderScope(child: MainApp()))
      |
      v
  MainApp.build() (main.dart:72)
      |
      |--> Shows SplashPage first
      |
      v
  SplashPage._navigate() (splash_page.dart:109)
      |
      |--> _waitForAuthRestore()             // Polls authControllerProvider.isLoading
      |        |                              // up to 50 times (5 seconds max)
      |        v
      |    AuthController.build() (auth_controller.dart:9)
      |        |
      |        |--> Starts with isLoading: true
      |        |--> Schedules _tryRestoreSession() via Future.microtask
      |        |
      |        v
      |    _tryRestoreSession() (auth_controller.dart:15)
      |        |
      |        |--> Checks: dataSource.isAuthenticated (is there a saved token?)
      |        |
      |        |--> YES: calls dataSource.getProfile()
      |        |       |
      |        |       |--> GET /profile with Bearer token
      |        |       |
      |        |       |--> SUCCESS: state = AuthState(user: user, isLoading: false)
      |        |       |             Connects WebSocket
      |        |       |
      |        |       |--> FAIL: Token expired. Calls signOut(), clears token.
      |        |                  state = AuthState(isLoading: false, user: null)
      |        |
      |        |--> NO: state = AuthState(isLoading: false, user: null)
      |
      v
  SplashPage checks auth state + onboarding flag
      |
      |--> user != null             --> MainNavPage (home)
      |--> user == null, seen onboarding --> LoginPage
      |--> user == null, first launch    --> OnboardingScreen
      |
      v
  _AuthGate (main.dart:103)
      |
      |--> ref.watch(authControllerProvider)
      |
      |--> isLoading?  --> Loading spinner
      |--> user != null? --> MainNavPage
      |--> user == null?  --> LoginPage
```

### Key Files in the Auth Flow

**AuthState** (`mobile/lib/features/auth/presentation/controllers/auth_state.dart`):
```dart
class AuthState {
  final bool isLoading;
  final AuthUser? user;
  final String? errorMessage;
  // ...
  // Note the clearUser flag in copyWith -- this is how signOut nullifies user
  // without Dart's type system fighting you on null assignment.
  AuthState copyWith({bool? isLoading, AuthUser? user, String? errorMessage, bool clearUser = false}) {
    return AuthState(
      isLoading: isLoading ?? this.isLoading,
      user: clearUser ? null : (user ?? this.user),  // <-- clearUser trick
      errorMessage: errorMessage,
    );
  }
}
```

**AuthController** (`mobile/lib/features/auth/presentation/controllers/auth_controller.dart`):
```dart
class AuthController extends Notifier<AuthState> {
  @override
  AuthState build() {
    // Start loading immediately so the splash screen knows to wait
    Future.microtask(() => _tryRestoreSession());
    return AuthState.initial().copyWith(isLoading: true);
  }
  // signIn, signUp, signOut, refreshProfile ...
}
```

Notice that `AuthController extends Notifier<AuthState>`, NOT `AsyncNotifier`. The
state is synchronous (`AuthState`), but the controller performs async work internally.
This is a deliberate pattern: it gives full control over loading/error states via
the `isLoading` and `errorMessage` fields rather than relying on `AsyncValue`.

**Provider wiring** (`mobile/lib/features/auth/presentation/providers/auth_providers.dart`):
```dart
final authControllerProvider = NotifierProvider<AuthController, AuthState>(
  AuthController.new,
);
```

### Sign-Out and Cache Invalidation

When the user signs out (`auth_controller.dart:82-98`), the controller:

1. Sets `isLoading: true`
2. Calls `dataSource.signOut()` (clears JWT from SharedPreferences)
3. Disconnects WebSocket
4. **Invalidates cached data** for the previous user:
   ```dart
   ref.invalidate(carsProvider);
   ref.invalidate(favoriteCarsProvider);
   ref.invalidate(favoriteIdsProvider);
   ```
5. Sets state to `clearUser: true` (nullifies user)

This invalidation is critical -- without it, the next user who logs in would briefly
see stale data from the previous session.

---

## 5. API Integration Pattern

### Architecture

```
  +------------------+        +-------------------+        +----------------+
  |   Widget/Page    | -----> |   DataSource      | -----> |   ApiClient    |
  |  (ref.watch)     |        |  (domain logic)   |        |  (HTTP layer)  |
  +------------------+        +-------------------+        +-------+--------+
                                                                   |
                                                           +-------v--------+
                                                           | Rust Backend   |
                                                           | (Render.com)   |
                                                           +----------------+
```

### ApiClient: The HTTP Singleton

**File:** `mobile/lib/core/services/api_client.dart`

The `ApiClient` is a **singleton** (factory constructor, line 9-11):

```dart
class ApiClient {
  static final ApiClient _instance = ApiClient._internal();
  factory ApiClient() => _instance;
  ApiClient._internal();
```

Every `ApiClient()` call anywhere in the app returns the same instance. This means
the token set during login is automatically available to every subsequent request.

### Token Injection

Every request method calls `_headers()` (line 56-64):

```dart
Map<String, String> _headers({bool auth = true}) {
  final headers = <String, String>{
    'Content-Type': 'application/json',
  };
  if (auth && _token != null) {
    headers['Authorization'] = 'Bearer $_token';
  }
  return headers;
}
```

Key detail: the `auth` parameter defaults to `true`. For unauthenticated endpoints
(like sign-in), the datasource passes `auth: false`:

```dart
// In api_auth_datasource.dart, line 23-27:
final response = await _client.post(
  '/auth/signin',
  body: {'email': email, 'password': password},
  auth: false,  // <-- No token for login
);
```

### Request/Response Cycle

```
  Widget calls ref.watch(carsProvider)
      |
      v
  FutureProvider executes:
      dataSource.getCars()
      |
      v
  ApiCarDataSource calls:
      _client.get('/cars')
      |
      v
  ApiClient.get():
      1. Build URL:  "https://qent-backend.onrender.com/api/cars"
      2. Log request: "[Qent API] > GET .../cars"
      3. Start stopwatch
      4. http.get(url, headers: {Authorization: Bearer <token>})
      5. Parse JSON response into ApiResponse
      6. Log response: "[Qent API] OK GET .../cars -> 200 (142ms)"
      7. Return ApiResponse
      |
      v
  DataSource checks response.isSuccess:
      YES --> Parse body into domain models (List<Car>)
      NO  --> throw Exception(response.errorMessage)
      |
      v
  FutureProvider resolves:
      AsyncValue.data(List<Car>) --> Widget rebuilds
      -- or --
      AsyncValue.error(exception) --> Widget shows error state
```

### ApiResponse Wrapper

```dart
class ApiResponse {
  final int statusCode;
  final dynamic body;

  bool get isSuccess => statusCode >= 200 && statusCode < 300;

  String get errorMessage {
    // Tries body['error'], then body['errors'], then generic message
  }
}
```

### Error Handling Pattern

Every HTTP method (`get`, `post`, `put`, `delete`) has a try/catch. On network
failure, it returns `ApiResponse(statusCode: 0, body: {'error': e.toString()})`
instead of throwing. This means the **datasource** decides whether to throw or
handle gracefully.

---

## 6. State Management Patterns

### Pattern 1: FutureProvider for One-Shot Data

Use `FutureProvider` when you need to fetch data once and cache it until invalidated.

**Example: Car listings** (`mobile/lib/features/home/presentation/providers/car_providers.dart:8`)

```dart
final carsProvider = FutureProvider<List<Car>>((ref) async {
  final dataSource = ref.watch(apiCarDataSourceProvider);
  return dataSource.getCars();
});
```

Usage in a widget:

```dart
final carsAsync = ref.watch(carsProvider);

carsAsync.when(
  data: (cars) => ListView.builder(...),
  loading: () => CircularProgressIndicator(),
  error: (e, st) => Text('Error: $e'),
);
```

**Example: Dashboard stats** (`mobile/lib/features/dashboard/presentation/providers/dashboard_providers.dart:80`)

```dart
final hostStatsProvider = FutureProvider<HostStats>((ref) async {
  final client = ref.watch(apiClientProvider);
  final response = await client.get('/dashboard/stats');
  if (response.isSuccess) {
    return HostStats.fromJson(response.body);
  }
  throw Exception(response.errorMessage);
});
```

### Pattern 2: FutureProvider.family for Parameterized Queries

Use `.family` when the same provider type needs different data based on a parameter.

**Example: Single car by ID** (`car_providers.dart:20`)

```dart
final carProvider = FutureProvider.family<Car?, String>((ref, carId) async {
  final dataSource = ref.watch(apiCarDataSourceProvider);
  return dataSource.getCar(carId);
});
```

**Example: Messages by conversation ID** (`chat_controller.dart:34`)

```dart
final messagesProvider = FutureProvider.family<List<ChatMessage>, String>(
  (ref, conversationId) async {
    final dataSource = ref.watch(apiChatDataSourceProvider);
    return dataSource.getMessages(conversationId);
  },
);
```

**Example: Homepage sections with coordinates** (`car_providers.dart:14`)

```dart
final homepageCarsProvider = FutureProvider.family<
  Map<String, List<Car>>,
  ({double? lat, double? lng})
>((ref, coords) async {
  final dataSource = ref.watch(apiCarDataSourceProvider);
  return dataSource.getHomepage(latitude: coords.lat, longitude: coords.lng);
});
```

Note the use of a **Dart record** `({double? lat, double? lng})` as the family
parameter. Records give you a lightweight, hashable parameter without creating a
dedicated class.

### Pattern 3: Notifier for Complex State Machines

Use `Notifier` (or `AsyncNotifier`) when you need:
- Multiple methods that modify state
- Loading/error tracking within the state
- Cross-provider coordination via `ref`

**Example: AuthController** (`auth_controller.dart`)

```dart
class AuthController extends Notifier<AuthState> {
  @override
  AuthState build() { /* initial state + session restore */ }

  Future<void> signIn({...}) async { /* sets loading, calls API, updates state */ }
  Future<void> signUp({...}) async { /* same pattern */ }
  Future<void> signOut() async { /* clears state, invalidates caches */ }
  Future<void> refreshProfile() async { /* re-fetches profile from API */ }
}
```

**Example: SearchController** (`mobile/lib/features/search/presentation/controllers/search_controller.dart`)

```dart
class SearchController extends Notifier<SearchState> {
  @override
  SearchState build() => SearchState(filters: SearchFilters());

  void updateBrandFilter(String brand) { /* updates state.filters */ }
  void updateSearchQuery(String query) { /* updates state.filters */ }
  void updatePriceRange(RangeValues priceRange) { /* updates state.filters */ }
  void clearAllFilters() { /* resets to defaults */ }
  // ... 8 more filter update methods
}
```

The `SearchController` is a great example of Notifier managing UI state. Each
filter update replaces the immutable `SearchState`, which triggers the
`filteredCarsProvider` to re-fetch since it `ref.watch`es the search controller.

### Pattern 4: Notifier for Optimistic Updates

**Example: FavoriteIdsNotifier** (`car_providers.dart:32-77`)

```dart
class FavoriteIdsNotifier extends Notifier<Set<String>> {
  Future<void> toggle(String carId) async {
    final wasFavorited = state.contains(carId);

    // 1. OPTIMISTIC UPDATE -- instant UI feedback
    if (wasFavorited) {
      state = Set<String>.from(state)..remove(carId);
    } else {
      state = Set<String>.from(state)..add(carId);
    }

    try {
      // 2. SYNC WITH SERVER
      final serverResult = await dataSource.toggleFavorite(carId);

      // 3. RECONCILE if server disagrees
      if (serverResult && !state.contains(carId)) {
        state = Set<String>.from(state)..add(carId);
      }
    } catch (e) {
      // 4. REVERT on error
      if (wasFavorited) {
        state = Set<String>.from(state)..add(carId);
      } else {
        state = Set<String>.from(state)..remove(carId);
      }
    }
  }
}
```

This is a textbook optimistic update pattern:
1. Update UI immediately
2. Send request to server
3. Reconcile if server state differs
4. Revert if request fails

### Pattern 5: Provider for Service Objects

Use plain `Provider` for objects that expose methods but don't hold reactive state.

**Example: ChatController as a service** (`chat_controller.dart:110`)

```dart
final chatControllerProvider = Provider<ChatController>((ref) {
  final dataSource = ref.watch(apiChatDataSourceProvider);
  return ChatController(dataSource, ref);
});
```

The `ChatController` is not a `Notifier` -- it is a plain class accessed via
`ref.read(chatControllerProvider)`. Its methods trigger invalidation of other
providers:

```dart
Future<void> sendMessage({required String chatId, ...}) async {
  await _dataSource.sendMessage(chatId, message);
  // Refresh messages and conversation list after sending
  _ref.invalidate(messagesStreamProvider(chatId));
  _ref.invalidate(messagesProvider(chatId));
  _ref.invalidate(chatsStreamProvider);
  _ref.invalidate(chatsProvider);
}
```

### How Invalidation / Refresh Works

```
  User taps "Send Message"
      |
      v
  ref.read(chatControllerProvider).sendMessage(chatId: "abc", ...)
      |
      v
  API call: POST /conversations/abc/messages
      |
      v
  ref.invalidate(messagesProvider("abc"))
      |
      +--> Next time any widget calls ref.watch(messagesProvider("abc")),
      |    the FutureProvider re-executes its async function.
      |    If a widget is currently watching it, it rebuilds immediately.
      |
      v
  Widget rebuilds with fresh data from the server
```

### Pattern 6: Reactive Provider Chains

**Example: filteredCarsProvider** (`search_providers.dart:21-110`)

This is the most sophisticated provider in Qent. It watches three other providers
and re-computes whenever any of them change:

```
  searchControllerProvider ----+
                               |
  apiCarDataSourceProvider ----+--> filteredCarsProvider (FutureProvider)
                               |
  userLocationProvider --------+
```

```dart
final filteredCarsProvider = FutureProvider<List<Car>>((ref) async {
  final searchState = ref.watch(searchControllerProvider);  // reactive!
  final filters = searchState.filters;
  final dataSource = ref.watch(apiCarDataSourceProvider);   // reactive!

  // ... maps filters to API query params ...
  final cars = await dataSource.searchCars(
    location: filters.location,
    minPrice: minPrice,
    // ... etc
  );

  // ... client-side filtering for things the API doesn't support ...
  return filtered;
});
```

When the user changes a filter (e.g., brand), this happens:

```
  User taps "BMW"
      |
      v
  ref.read(searchControllerProvider.notifier).updateBrandFilter("BMW")
      |
      v
  searchControllerProvider state changes
      |
      v
  filteredCarsProvider is watching searchControllerProvider
      --> Automatically re-executes
      --> Calls searchCars(make: "BMW")
      --> Returns new filtered list
      |
      v
  SearchPage widget is watching filteredCarsProvider
      --> Rebuilds with new car list
```

### Pattern 7: Dependent Providers

**Example: Notifications depending on auth** (`notification_providers.dart:28-35`)

```dart
final currentUserNotificationsProvider = StreamProvider<List<NotificationModel>>((ref) {
  final userId = ref.watch(authControllerProvider).user?.uid;
  if (userId == null) {
    return Stream.value([]);  // No user = empty notifications
  }
  final repository = ref.watch(notificationRepositoryProvider);
  return repository.getNotifications(userId);
});
```

This provider re-subscribes to the notification stream whenever the user changes
(login/logout/switch user). If `userId` is null, it short-circuits to an empty stream.

---

## 7. User Preferences with SharedPreferences

Qent uses `SharedPreferences` for three purposes:

### 7.1 JWT Token Persistence

**File:** `mobile/lib/core/services/api_client.dart`

```dart
// On initialize (app startup) -- line 22-27:
Future<void> initialize({String? baseUrl}) async {
  _prefs = await SharedPreferences.getInstance();
  _token = _prefs?.getString('auth_token');  // <-- Restore saved token
}

// On login -- line 36-39:
Future<void> setToken(String token) async {
  _token = token;
  await _prefs?.setString('auth_token', token);  // <-- Persist token
}

// On logout -- line 49-53:
Future<void> clearToken() async {
  _token = null;
  await _prefs?.remove('auth_token');  // <-- Remove token
}
```

**Flow:**
```
  App Launch --> SharedPreferences.getString('auth_token')
      |
      |--> Token exists? --> Set _token, try GET /profile
      |                          |
      |                          |--> 200 OK --> User is logged in
      |                          |--> 401    --> Token expired, clearToken()
      |
      |--> No token? --> Show login screen
```

### 7.2 Onboarding Flag

**File:** `mobile/lib/features/onboarding/presentation/pages/onboarding_screen.dart:111`

```dart
// When user finishes onboarding:
SharedPreferences.getInstance().then((prefs) {
  prefs.setBool('has_seen_onboarding', true);
});
```

**File:** `mobile/lib/features/splash/presentation/pages/splash_page.dart:119`

```dart
// During splash navigation:
final prefs = await SharedPreferences.getInstance();
final hasSeenOnboarding = prefs.getBool('has_seen_onboarding') ?? false;
```

### 7.3 WebSocket Token

**File:** `mobile/lib/core/services/websocket_service.dart:37-39`

```dart
final prefs = await SharedPreferences.getInstance();
final token = prefs.getString('token');
```

Note: The WebSocket service reads from key `'token'` while ApiClient uses
`'auth_token'`. This is worth noting -- if these get out of sync, WebSocket
connections will fail silently.

### Summary of SharedPreferences Keys

| Key                    | Type   | Set Where                    | Read Where                   |
|------------------------|--------|------------------------------|------------------------------|
| `auth_token`           | String | ApiClient.setToken()         | ApiClient.initialize()       |
| `has_seen_onboarding`  | bool   | OnboardingScreen             | SplashPage._navigate()       |
| `token`                | String | (see note)                   | WebSocketService.connect()   |

---

## 8. Real-time with WebSocket + StreamProvider

### WebSocket Architecture

**File:** `mobile/lib/core/services/websocket_service.dart`

```
  +-------------------+        WSS         +------------------+
  | Flutter App        | <===============> | Rust Backend      |
  | WebSocketService   |                   | /ws?token=JWT     |
  +-------------------+                    +------------------+
        |
        |  StreamController<WsEvent>.broadcast()
        |
        v
  +-------------------+     +-------------------+     +-------------------+
  | Chat listeners    |     | Call listeners    |     | Typing listeners  |
  | (message updates) |     | (WebRTC signals)  |     | (typing dots)     |
  +-------------------+     +-------------------+     +-------------------+
```

### WsEvent Model

```dart
class WsEvent {
  final String type;               // "chat_message", "typing", "call_offer", etc.
  final Map<String, dynamic> payload;
}
```

### Connection Lifecycle

```
  AuthController.signIn() / _tryRestoreSession()
      |
      v
  ref.read(wsServiceProvider).connect()
      |
      v
  WebSocketService.connect():
      1. Guard: if already connecting/connected, return
      2. Read token from SharedPreferences
      3. Convert API URL to WebSocket URL:
           https://qent-backend.onrender.com/api
           --> wss://qent-backend.onrender.com/ws?token=JWT
      4. WebSocketChannel.connect(wsUrl)
      5. await channel.ready
      6. Start heartbeat (ping every 25s)
      7. Listen to stream:
           - Parse JSON messages into WsEvent
           - Broadcast via _eventController
           - On error/done: _handleDisconnect()
```

### Auto-Reconnect with Exponential Backoff

```dart
void _handleDisconnect() {
  _state = WsState.disconnected;
  _heartbeatTimer?.cancel();

  // Exponential backoff: 1s, 2s, 4s, 8s, 16s, then cap at 30s
  final delay = (_reconnectAttempts < 5)
      ? (1 << _reconnectAttempts)  // 2^n
      : _maxReconnectDelay;        // 30 seconds
  _reconnectAttempts++;

  _reconnectTimer = Timer(Duration(seconds: delay), () => connect());
}
```

```
  Disconnect
      |
      v
  Attempt 0: wait 1s  --> connect()  --> fail?
  Attempt 1: wait 2s  --> connect()  --> fail?
  Attempt 2: wait 4s  --> connect()  --> fail?
  Attempt 3: wait 8s  --> connect()  --> fail?
  Attempt 4: wait 16s --> connect()  --> fail?
  Attempt 5+: wait 30s --> connect() --> ...
```

### WebSocket Event Types

The service provides typed send methods for different event categories:

| Method                | WS Type          | Purpose                        |
|-----------------------|------------------|--------------------------------|
| `sendChatMessage()`   | `chat_message`   | Send a text/media message      |
| `sendTyping()`        | `typing`         | Typing indicator               |
| `sendCallOffer()`     | `call_offer`     | WebRTC SDP offer               |
| `sendCallAnswer()`    | `call_answer`    | WebRTC SDP answer              |
| `sendIceCandidate()`  | `ice_candidate`  | WebRTC ICE candidate           |
| `sendCallReject()`    | `call_reject`    | Decline incoming call          |
| `sendCallHangup()`    | `call_hangup`    | End active call                |

### StreamProviders for Real-Time Data

**Online status** (`online_status_providers.dart`):

```dart
final onlineStatusStreamProvider = StreamProvider.family<bool, String>(
  (ref, userId) async* {
    yield false;  // Stub -- would consume wsServiceProvider.events
  },
);
```

**Chat conversations** (`chat_controller.dart:22`):

```dart
final chatsStreamProvider = StreamProvider<List<Chat>>((ref) async* {
  final dataSource = ref.watch(apiChatDataSourceProvider);
  try {
    final chats = await dataSource.getConversations();
    yield chats;
  } catch (e) {
    yield [];
  }
});
```

Currently, Qent's `StreamProvider`s wrap `FutureProvider` data (one-shot yield).
To make them truly real-time, you would integrate `wsServiceProvider.events` and
yield updated data whenever a relevant WebSocket event arrives.

---

## 9. Common Patterns & Anti-patterns

### ref.watch() vs ref.read()

```
  DO:   In build() methods, always use ref.watch()
        --> Widget rebuilds when data changes

  DO:   In callbacks (onTap, onPressed), use ref.read()
        --> Read current value once, don't subscribe

  DON'T: Use ref.watch() in callbacks
        --> Creates subscriptions that outlive the callback
```

**Correct examples from Qent:**

```dart
// WATCH in build -- SearchPage (search_page.dart:44-49)
Widget build(BuildContext context) {
  final carsAsync = ref.watch(filteredCarsProvider);     // watch = rebuild
  final searchState = ref.watch(searchControllerProvider); // watch = rebuild
  final carController = ref.read(carControllerProvider);   // read = action object
  // ...
}

// READ in callback -- SearchPage (search_page.dart:36)
void _onSearchChanged() {
  ref.read(searchControllerProvider.notifier).updateSearchQuery(
    _searchController.text,
  );
}
```

### When to Invalidate

Invalidate a provider when the underlying data has changed due to a mutation:

```dart
// GOOD: After sending a message, refresh the message list
_ref.invalidate(messagesProvider(chatId));
_ref.invalidate(chatsProvider);

// GOOD: After logout, clear user-specific caches
ref.invalidate(carsProvider);
ref.invalidate(favoriteCarsProvider);
ref.invalidate(favoriteIdsProvider);

// GOOD: Pull-to-refresh
onRefresh: () async {
  ref.invalidate(filteredCarsProvider);
  await ref.read(filteredCarsProvider.future);
},
```

### Provider Scoping and Dependencies

Qent follows a clean dependency chain:

```
  ApiClient (singleton)
      |
      v
  DataSource (Provider, depends on ApiClient)
      |
      v
  FutureProvider / Notifier (depends on DataSource)
      |
      v
  Widget (watches provider)
```

**Anti-pattern to avoid:** Don't create circular dependencies between providers.
If Provider A watches Provider B and Provider B watches Provider A, Riverpod
will throw a `ProviderLoopError`.

### Immutable State Updates

Qent always creates **new objects** when updating state, never mutates in place:

```dart
// GOOD (from FavoriteIdsNotifier):
state = Set<String>.from(state)..remove(carId);  // New set

// BAD (would not trigger rebuild):
state.remove(carId);  // Mutating existing set -- Riverpod won't detect it
```

### Provider Organization

Qent organizes providers by feature:

```
mobile/lib/
  features/
    auth/
      presentation/
        providers/auth_providers.dart      <-- All auth-related providers
        controllers/auth_controller.dart   <-- AuthController class
        controllers/auth_state.dart        <-- AuthState class
    home/
      presentation/
        providers/car_providers.dart       <-- Car-related providers
    search/
      presentation/
        providers/search_providers.dart    <-- Search providers
        controllers/search_controller.dart <-- SearchController class
    chat/
      presentation/
        controllers/chat_controller.dart   <-- Chat providers + controller
        providers/online_status_providers.dart
        providers/stories_providers.dart
    dashboard/
      presentation/
        providers/dashboard_providers.dart <-- Host dashboard providers
```

### Error Handling Strategy

Qent uses a two-level error strategy:

1. **ApiClient level:** Never throws. Returns `ApiResponse(statusCode: 0, ...)` on
   network failure.
2. **DataSource level:** Checks `response.isSuccess` and throws `Exception` with
   the error message if false.
3. **Controller/Provider level:**
   - `FutureProvider`: Exception propagates to `AsyncValue.error`, widget handles it.
   - `Notifier`: Controller catches exception, sets `errorMessage` in state.

---

## 10. Exercises

These exercises use the actual Qent codebase. Try them in order.

### Exercise 1: Add a "Host Earnings This Week" Provider

Create a new `FutureProvider` in `dashboard_providers.dart` that fetches weekly
earnings. Follow the pattern of `hostStatsProvider`.

```dart
// Skeleton:
final hostWeeklyEarningsProvider = FutureProvider<double>((ref) async {
  final client = ref.watch(apiClientProvider);
  // TODO: Call the API and parse the response
  // Hint: look at how hostStatsProvider does it
});
```

### Exercise 2: Add Pull-to-Refresh to the Chat List

In the messages page, add a `RefreshIndicator` that invalidates `chatsProvider`:

```dart
RefreshIndicator(
  onRefresh: () async {
    ref.invalidate(chatsProvider);
    await ref.read(chatsProvider.future);
  },
  child: /* your existing list */,
)
```

### Exercise 3: Create a "Recently Viewed Cars" Provider

Using `Notifier`, create a provider that:
1. Maintains a `List<String>` of car IDs (max 10)
2. Has an `addViewed(String carId)` method
3. Deduplicates and keeps most recent first

```dart
class RecentlyViewedNotifier extends Notifier<List<String>> {
  @override
  List<String> build() => [];

  void addViewed(String carId) {
    // TODO: Remove if already present, add to front, cap at 10
    // Remember: create a NEW list, don't mutate state directly
  }
}
```

### Exercise 4: Make Online Status Real-Time

Replace the stub in `online_status_providers.dart` with a real implementation:

```dart
final onlineStatusStreamProvider = StreamProvider.family<bool, String>(
  (ref, userId) async* {
    final wsService = ref.watch(wsServiceProvider);
    await for (final event in wsService.events) {
      if (event.type == 'user_status' && event.payload['user_id'] == userId) {
        yield event.payload['is_online'] as bool;
      }
    }
  },
);
```

### Exercise 5: Add a StateProvider for Theme Mode

Create a simple `StateProvider` for toggling dark/light theme:

```dart
final themeModeProvider = StateProvider<ThemeMode>((ref) => ThemeMode.dark);
```

Then use it in `MainApp`:

```dart
final themeMode = ref.watch(themeModeProvider);
return MaterialApp(
  themeMode: themeMode,
  // ...
);
```

Bonus: Persist the choice to `SharedPreferences` and restore it on startup.

### Exercise 6: Trace a Full Data Flow

Pick `hostPendingBookingsProvider` and trace the complete flow:

1. Find where the provider is defined (file + line)
2. Find which widget watches it
3. Follow the API call path: Provider -> ApiClient -> Backend endpoint
4. Identify what would trigger a refresh
5. Draw the dependency graph

This exercise builds your ability to navigate the codebase and understand how
Riverpod providers connect.

---

## Quick Reference Card

```
+---------------------------------------------------------------+
|                  RIVERPOD CHEAT SHEET                          |
+---------------------------------------------------------------+
|                                                               |
|  DECLARE:                                                     |
|    final myProvider = Provider<T>((ref) => value);            |
|    final myFuture = FutureProvider<T>((ref) async => value);  |
|    final myNotifier = NotifierProvider<N, T>(N.new);          |
|                                                               |
|  USE:                                                         |
|    ref.watch(provider)           -- subscribe (in build)      |
|    ref.read(provider)            -- one-shot (in callbacks)   |
|    ref.read(provider.notifier)   -- access Notifier methods   |
|    ref.invalidate(provider)      -- force re-compute          |
|    ref.listen(provider, (p,n){}) -- side effects              |
|                                                               |
|  ASYNC VALUE:                                                 |
|    asyncValue.when(                                           |
|      data: (d) => Widget,                                     |
|      loading: () => Widget,                                   |
|      error: (e,st) => Widget,                                 |
|    )                                                          |
|                                                               |
|  FAMILY (parameterized):                                      |
|    FutureProvider.family<T, Param>((ref, param) async => ...) |
|    ref.watch(myProvider(param))                               |
|                                                               |
|  SETUP:                                                       |
|    main() => runApp(ProviderScope(child: MyApp()))            |
|    class MyPage extends ConsumerWidget { ... }                |
|    class MyPage extends ConsumerStatefulWidget { ... }        |
|                                                               |
+---------------------------------------------------------------+
```
