# Tests and models of what others do

## Inputs and items

That one failing item takes nothing from the items around it can fail two ways, stopping at the failure or discarding what came before it, so its test puts the failing item between two that succeed and asserts the results of both; a failure at either end of the sequence shows one wrong form only. A test of a rule over several kinds of input gives one input of each kind. Before you rely on such a test, run it once against each wrong form and watch it fail.

## A model of a tool

Code or a test that models what a tool does with an input — its output, its error text, its formatting, the place it reports — takes the expected value, the parser or the stand-in from the tool itself: run it on a real input of each kind the model covers and read what it prints, never write it from memory or from the tool's documentation. Why: a model written from expectation holds what its author believes, and the tool's output decides otherwise for the inputs that matter.

## A third party's form and server

Before a design rests on what a third party's UI lets a user enter, or on what its server does on a write, read that form and its submit code and that server method, defaults included, at the version the project runs. Why: a third party's API, UI and notifications each show a different part of its behaviour, and only its code at that version shows which part the design meets.

## A time bound for a guard

A test that a guard stops a costly input within a time bound sets the bound from the ratio of work between what the guard does before it stops and what the input costs without it: the bound tells a stop from an unguarded run on every machine where the stop takes less than the bound and more than the bound divided by that ratio. Set it at no less than five times the slowest CI run known then; afterwards a CI run is a finding only past half the bound, and only then is the bound set again. Why: a measured time is a sample of one machine, so the next slow runner makes a bound set from it false while the code stays the same.
