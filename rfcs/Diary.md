# Project Diary

- Feature Name: diary record
- Start Date: 2023-07-27 10:52:11

## Summary

To record the new technologies and concepts I'm learning. 

## Motivation

to adopt the professional habit of an engineer using English for documentation, I utilize this approach. It not only enhances my ability to articulate problems and express my thoughts, but it also cultivates my skills in pinpointing key points and communicating fluently.

## 07/26

it's start from I want to practice the reservation system and it can be used by a number of type of booking systems, Today I learned how to pre-prepared settings, for example, I could have .**github/workflow/build.yml** that automatically triggers the Continuous Integration (CI) process whenever I push or pull code. Also learned how to set up the pre-commit, it's a great way to run a bunch of commands when u type "git commit -a", that means I can write some scripts like **cargo check**, **cargo deny**, **cargo nextest(further test)** and run all of these at the same time.

## 07/27

Today, I am in the process of developing a concurrent reservation system, where a primary challenge is conflict avoidance. Initially, I considered resolving this issue at the application level by comparing the time slots in the database with those selected by the user to ensure availability. However, this approach has a significant drawback: if User A is contemplating a reservation for a particular time slot, User B might secure that slot first, preventing A from booking.

In this situation, even if User A could initially book a time slot, a conflict arises because User B submitted the request sooner. Regardless of how I tackle this problem at the application level, complex locking issues seem inevitable, leading to considerable overhead.

Consequently, I am contemplating utilizing PostgreSQL's EXCLUDE constraints as a solution, applying restrictions directly at the database core level. This way, I can ensure data consistency at the database level and bypass the complexity at the application level. This method incurs less overhead and effectively avoids concurrent conflicts.

```sql
SELECT int4range(10,20) && int4range(1, 9);
```
**that return false, cuz that isn't overlapping**