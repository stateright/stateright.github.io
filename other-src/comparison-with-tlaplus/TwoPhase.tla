------------------------------- MODULE TwoPhase ----------------------------- 
(*
This module implements a subset of the two phase commit specification presented
in the paper "Consensus on Transaction Commit" by Jim Gray and Leslie Lamport.
It has been adapted only slightly for the book.

The paper can be found here:
https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2003-96.pdf

From that paper:

> This paper appeared in _ACM Transactions on Database Systems_, Volume 31,
> Issue 1, March 2006 (pages 133-160).  This version should differ from the
> published one only in formatting, except that it corrects one minor error on
> the last page.
>
> Copyright 2005 by the Association for Computing Machinery, Inc.  Permission
> to make digital or hard copies of part or all of this work for personal or
> classroom use is granted without fee provided that copies are not made or
> distributed for profit or  commercial  advantage and  that  copies  bear this
> notice  and  the  full  citationon the first page.  Copyrights for components
> of this work owned by others than ACM must be honored.  Abstracting with
> credit is permitted.  To copy otherwise, to republish, to post on servers, or
> to redistribute to lists, requires prior specificpermission and/or a fee.
> Request permissions from Publications Dept, ACM Inc.,fax +1 (212) 869-0481,
> or permissions@acm.org.
*)

(* ANCHOR: constants *)
CONSTANT RM    
(* ANCHOR_END: constants *)

(* ANCHOR: variables *)
VARIABLES
  rmState,
  tmState,
  tmPrepared,
  msgs           
(* ANCHOR_END: variables *)

(* ANCHOR: types *)
Message ==
  [type : {"Prepared"}, rm : RM]  \cup  [type : {"Commit", "Abort"}]
   
TypeOK ==  
  /\ rmState \in [RM -> {"working", "prepared", "committed", "aborted"}]
  /\ tmState \in {"init", "committed", "aborted"}
  /\ tmPrepared \subseteq RM
  /\ msgs \subseteq Message
(* ANCHOR_END: types *)

(* ANCHOR: init *)
Init ==   
  /\ rmState = [rm \in RM |-> "working"]
  /\ tmState = "init"
  /\ tmPrepared   = {}
  /\ msgs = {}
(* ANCHOR_END: init *)

(* ANCHOR: next *)
TMCommit ==
  /\ tmState = "init"
  /\ tmPrepared = RM
  /\ tmState' = "committed"
  /\ msgs' = msgs \cup {[type |-> "Commit"]}
  /\ UNCHANGED <<rmState, tmPrepared>>

TMAbort ==
  /\ tmState = "init"
  /\ tmState' = "aborted"
  /\ msgs' = msgs \cup {[type |-> "Abort"]}
  /\ UNCHANGED <<rmState, tmPrepared>>

TMRcvPrepared(rm) ==
  /\ tmState = "init"
  /\ [type |-> "Prepared", rm |-> rm] \in msgs
  /\ tmPrepared' = tmPrepared \cup {rm}
  /\ UNCHANGED <<rmState, tmState, msgs>>

RMPrepare(rm) == 
  /\ rmState[rm] = "working"
  /\ rmState' = [rmState EXCEPT ![rm] = "prepared"]
  /\ msgs' = msgs \cup {[type |-> "Prepared", rm |-> rm]}
  /\ UNCHANGED <<tmState, tmPrepared>>
  
RMChooseToAbort(rm) ==
  /\ rmState[rm] = "working"
  /\ rmState' = [rmState EXCEPT ![rm] = "aborted"]
  /\ UNCHANGED <<tmState, tmPrepared, msgs>>

RMRcvCommitMsg(rm) ==
  /\ [type |-> "Commit"] \in msgs
  /\ rmState' = [rmState EXCEPT ![rm] = "committed"]
  /\ UNCHANGED <<tmState, tmPrepared, msgs>>

RMRcvAbortMsg(rm) ==
  /\ [type |-> "Abort"] \in msgs
  /\ rmState' = [rmState EXCEPT ![rm] = "aborted"]
  /\ UNCHANGED <<tmState, tmPrepared, msgs>>

Next ==
  \/ TMCommit \/ TMAbort
  \/ \E rm \in RM : 
       TMRcvPrepared(rm) \/ RMPrepare(rm) \/ RMChooseToAbort(rm)
         \/ RMRcvCommitMsg(rm) \/ RMRcvAbortMsg(rm)
(* ANCHOR_END: next *)

(* ANCHOR: spec *)
Spec == Init
     /\ [][Next]_<<rmState, tmState, tmPrepared, msgs>>
(* ANCHOR_END: spec *)

(* ANCHOR: properties *)
Consistent ==
  \A rm1, rm2 \in RM : ~ /\ rmState[rm1] = "aborted"
                         /\ rmState[rm2] = "committed"
(* ANCHOR_END: properties *)
=============================================================================
