# Project Diary

- Feature Name: diary record
- Start Date: 2023-07-27 10:52:11

## Summary

To document the novel technologies and concepts that I am mastering.

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

## 08/01

### Title: build the functions about reservation(1)

learned how to build the trait about what function I need to implement. some basic function like reserve/delete/update etc. to be continued...

## 08/02

### Title: build the functions about reservation(2)

While planning the necessary functions for ReservationManager, I encountered various issues such as a lack of time scope and time conversion (from timestamp to chrono::Utc), but I'm persisting and making progress. by the way, today's **BIGGEST** part is HOW can I deal with the database "status"(postgres) type can not convert to ReservationStatus.**(finished)**

## 08/03

### Title: build the functions about reservation(3)

Today I add new notes to documenting some error when I make this project also changed some README.md, and next time I will be implementing the remaining functions.

- [x] TODO: I need to fix my git... .pre-commit-config.yaml

## 08/04

### Title: pre-commit problem

I need to check my configuration to see where the problem is, but I don't know how to do it. I'll try to figure it out a several days, before I solve this problem I should to move on to next step, to implement remaining functions.

## 08/06

### Title: due to unittest so build mock database to do some connection

First and foremost, I've adopted the good practice of using Test-Driven Development (TDD) to ensure my code during refactoring, with the aim of avoiding potential problems that may arise when communicating with external systems. In this process, I've encountered two main challenges:

Testing the Database Interaction: Previously, when testing the network layer in unit tests, we could mock requests and self-answer. However, now we need to interact with the database, requiring a more authentic data source. To resolve this, I've created a mock database to perform a series of migrations during testing. This approach allows us to operate on a relatively real database without interfering with the actual one, then delete the mock database after satisfying the unit test.

Clear Conflict Messages with sqlx::PgDatabase::Error: While further testing for conflict reserves, I needed to understand the problem through error messages. However, PgDatabaseError didn't provide a way to return a very "clear" conflict problem to the user. Since PgDatabaseError doesn't offer a better method for detailed error information, we had to implement our own function to parse a large amount of raw data obtained from get_raw() within PgDatabaseError.

- [ ] TODO: write a parser

## 08/07

### Title: fixing cargo script dut to target conflict problem

The issue I encountered is that when using four cargo scripts, they conflict with each other due to the previously compiled files. This leads to the 'file were modified by this hook' error, as the next test requires recompilation to meet the script's criteria. Currently, I'm storing their respective compiled files separately, but I feel this isn't an ideal solution, and I will continue to look for a better way.

Ps: If I fixed I'll document it to [Error note](./error-notes.md)

## 08/08

### Title: write a parser to catch errors and describe them as detailed as possible

1. Printing Details of a Specific Error: To print the details of a particular error, the ReservationConflictInfo enum is used, encompassing two cases: successful (Parsed) and unsuccessful (Unparsed).
2. Structure for Successful Printing: Successful printing utilizes the ReservationConflict structure, containing two separate structures to represent two kinds of Reserve scenarios, encapsulated by ReservationWindow.
3. Newly Added Parsing Structure: A new structure, ParsedInfo, was added to hold parsed paired HashMap data. Regular expressions, .try_fold(), and .and_then() methods were applied for parsing.
4. Error Handling Approach: Reflected on the solutions from a previous todo project, gaining insights into managing errors in the current project.

- **TODO:familiar my function**
- **TODO:write some error handling**
