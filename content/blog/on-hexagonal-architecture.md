---
title: "On Hexagonal Architecture"
description: "Why a hexagonal architecture can be better than traditional MVC-style architecture, with a real-world example."
published: "2018-09-02"
location: "Frankfurt, Germany"
---

> A good architect maximises the number of decisions not made
>
> — Robert C. Martin in Clean Architecture

Most web services I worked with use a MVC-style architecture, with a `handlers` package and, if at all, a `repository` package. While this may be great for small services, the `handlers` package introduces a big problem: It mixes transport logic with business logic. This makes refactoring hard (imagine switching your HTTP framework) and therefore forces you to make decisions about these kind of things before even starting the project. So when I started a new project recently, I decided to use the hexagonal architecture (aka _Ports and Adapters_) and so far I'm really happy.

<!-- more -->

## What I like about it

Here's what I like about it:

### All dependencies point inward

All outer layers like transport, storage or logging depend on the business logic, but never the other way around. The business logic is agnostic of any other layer, it doesn't care, how it's served or how data is stored, it's pure code. This makes changing it super simple.

### You can defer decisions

You can defer many decisions about technologies used until you really need them. You could, for example, start out with an `inmem` package for storage and only decide which database to use when you really need persistence.


## Refactorings are simple

Since everything is contained in it's domain, refactoring the transport package, for example, is refactoring only transport code. There is pure separation of concerns and everything has a clear place.

### Testing business logic is simple

Since you only have pure code without layer dependencies, you can easily inject an `inmem` package as storage for example. No need for mocking complex database structs, which cost time.

### Why a hexagon

It actually doesn't matter how many sides the shape has. The point is, that there are many. The center represents the business logic and every side represents a port into or out of our application (that's why it's also called _Ports and Adapters_).

## A real-world example

For this post let's assume that we're building an API to manage an inventory of some sort – it's still similar to the application I'm building. You should be able to list all items in the inventory and logged-in users should have basic CRUD access.

To set things up, I created these packages:

- `user`: Domain logic for user management (login, logout, validate)
- `inventory`: Domain logic for inventory (list, create, read, update, delete)
- `inmem`: In-memory storage for user and inventory
- `http`: HTTP transport logic, does de-/encoding and request handling

The `user` and `inventory` packages define an interface for the storage struct. This way we can inject any struct that implements that interface and keep the separation of concerns.

Here's how our `main.go` would look like, simplified:

```go
package main

import (
	"net/http"

	transport "github.com/bahlo/inventory/http"
	"github.com/bahlo/inventory/inmem"
	"github.com/bahlo/inventory/inventory"
	"github.com/bahlo/inventory/user"
)

func main() {
	userStorage := inmem.NewUserStorage()
	userService := user.NewService(userStorage)
	http.Handle("/v1/user", transport.UserHandler(userService))

	inventoryStorage := inmem.NewInventoryStorage()
	inventoryService := inventory.NewService(inventoryStorage)
	http.Handle("/v1/inventory", transport.InventoryHandler(inventoryService))

	http.ListenAndServe(":8080", nil)
}
```

## Problems

In real-world applications, we always have to make compromises and fight for clear separation of concerns. This is a list of problems I ran into while building the application.

### Authentication

Let's say we want to implement a token-based authentication and we need to protect some routes of the `inventory` service. Where should we get the token-validation function from? Getting the user service via our `NewService` constructor would result in unnecessary dependencies.
What I ended up doing was a `http.Guard(userService)` function, which returned a http middleware (`func (next http.Handler) http.Handler`) which parses the token and validates with `userService.UserForToken`. The middleware is then passed into the `transport.InventoryHandler` and wrapped around methods that needed protection. This way, there are no new dependencies.

### Model decorators

Another problem are Go tags ([read up](https://golang.org/ref/spec#Tag)). Let's say we need some for transport (e.g. `json:"id"`) and database (e.g. `db:"first_name"`), but they're attached to the model, which lies in the domain logic. A fix would be having the model structs duplicated in the other packages with the needed tags, but this introduces a lot of duplicate code and unnecessary complexity, so I decided to leave it as-is right now until I found a better solution.

## Conclusion

I'm really happy with my application and I don't think I'll go back to MVC-style applications any time soon.
I encourage you to try and build a service using a hexagonal architecture and share your experience – doesn't have to be Go. If you want to read more on hexagonal architecture, I recommend checking out [go-kit](https://gokit.io/). There is also a GopherCon 2018 talk by Matt King, here is [the script](https://about.sourcegraph.com/go/gophercon-2018-how-do-you-structure-your-go-apps) until the videos are up. Also if you have comments or suggestions, [hit me up](/contact) 🤙.
