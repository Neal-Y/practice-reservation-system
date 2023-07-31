# Project Diary

- Feature Name: diary record
- Start Date: 2023-07-27 10:52:11

## Summary

To record the new technologies and concepts I'm learning. 

## Motivation

to adopt the professional habit of an engineer using English for documentation, I utilize this approach. It not only enhances my ability to articulate problems and express my thoughts, but it also cultivates my skills in pinpointing key points and communicating fluently.

## 07/26

### Title: Intro With Pre-work

it's start from I want to practice the reservation system and it can be used by a number of type of booking systems, Today I learned how to pre-prepared settings, for example, I could have .**github/workflow/build.yml** that automatically triggers the Continuous Integration (CI) process whenever I push or pull code. Also learned how to set up the pre-commit, it's a great way to run a bunch of commands when u type "git commit -a", that means I can write some scripts like **cargo check**, **cargo deny**, **cargo nextest(further test)** and run all of these at the same time.

## 07/27

## Title: Core Logic

Today, I am in the process of developing a concurrent reservation system, where a primary challenge is conflict avoidance. Initially, I considered resolving this issue at the application level by comparing the time slots in the database with those selected by the user to ensure availability. However, this approach has a significant drawback: if User A is contemplating a reservation for a particular time slot, User B might secure that slot first, preventing A from booking.

In this situation, even if User A could initially book a time slot, a conflict arises because User B submitted the request sooner. Regardless of how I tackle this problem at the application level, complex locking issues seem inevitable, leading to considerable overhead.

Consequently, I am contemplating utilizing PostgreSQL's EXCLUDE constraints as a solution, applying restrictions directly at the database core level. This way, I can ensure data consistency at the database level and bypass the complexity at the application level. This method incurs less overhead and effectively avoids concurrent conflicts.

```sql
SELECT int4range(10,20) && int4range(1, 9);
```
**that return false, cuz that isn't overlapping**

## 07/28

### Title: Database Schema Intro

introducing the database "schema" is kind of like C/C++ "namespace" which means that can let all of set up packaged as a one object, make all items better manageable.

why I need to use "gist search" ? The main reason for this choice is that it allows the 'resource_id' to be compared while also executing commands to determine overlap. In other words, it gives me the capability to use **more than one command at the same time**, which is why I chose to use the GiST (Generalized Search Tree) approach. also we created a "reservation_changes" table which means when we have any changes and the trigger will save the change's details insert into the database.

Finally, we implemented a trigger function in PostgreSQL, tasked with logging CRUD operations to the 'reservation_changes' table. However, we must remain mindful of performance implications. High concurrency, brought on by a multitude of simultaneous connections, requires us to ensure that system stability isn't compromised by this logging process."

## 07/29

### Title: Start To Code

It's been a long time, finally I code some smell things that use tonic and gRPC, I use tonic-build crate in the build.rs file which can build the reservation.proto and then BOOM we have reservation.rs in the pb dir that is cuz it can converted protobuf into rs, it's same concept as "SCHEMA", also I learned that "tokei" which is a counting line of code application, that's it, it's wonderful day again. by the way, I'm traveling so maybe not to detail but I do my best : ）.

## 07/31

### Title: use sqlx build migration(遷移)

First, it's important to understand that 'sqlx' is an ORM (Object Relational Mapping) system crate, similar to SeaORM, but with a deeper integration with the database. Once familiar with this concept, I moved on to learning about database migrations. Migrations are used when changes to the database's schema are needed. I generally use the command 'sqlx migrate add 'filename' -r' for this purpose, where '-r' stands for revision migration. Each migration created has a unique timestamp. Every migration is composed of two files, 'up' and 'down'. The 'up' file executes the commands necessary to implement the migration, while the 'down' file allows for the migration to be rolled back if needed.