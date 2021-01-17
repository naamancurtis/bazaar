# Current Functionality

## Cart Management

As a customer, I want to be able to add items to my cart so that I can purchase
them.

As a customer, I want to be able to remove items from my cart so that I can purchase
them.

As a customer, I want to be able to view all the items in my cart and see how
much they would cost me, so that I can decide to purchase them or not

## Customer Management

As a customer, I want to be able to log in at any point and have the items I
have in my cart maintained, so that I don't have to go back and re-add them.

As a customer, I want to be able to log out of my account so that I know the
device I am on no longer has access to my account.

As a logged in customer, I want to be able to view my personal details so that I
can verify they're correct.

As a logged in customer, I want to be able to edit my personal details so that I
can keep them up to date.

## Authentication

_Breaking the User Story type requirements to explain what's going on here._

Authentication in the application has been implemented with Access & Refresh **JSON Web Tokens**.
Where **Access tokens** are short lived and irrevocable and **Refresh tokens** are longer lived and
revocable.

Both tokens are stored in `HttpOnly` cookies and subsequently are sent on every
request.
