Here’s an **exhaustive audit checklist** you can use (or adapt) for compliance when doing binary swaps / replacements (like GNU → uutils) especially in server / regulated environments. It combines general compliance requirements (PCI, HIPAA, FedRAMP, etc.) with Switchyard-relevant details.

---

## 🚦 Compliance Audit Checklist

Use this to verify that your binary replacement workflow is compliant. Mark each item YES/NO, evidence location, owner.

| Category                       | Control / Requirement                                                                                 | Why It Matters                                                       | Evidence / How to Validate                             |
| ------------------------------ | ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ------------------------------------------------------ |
| **Governance & Documentation** | There is a documented change management policy covering binary replacements.                          | Ensures controlled, auditable changes.                               | Policy doc; audit logs.                                |
|                                | There is a documented security roles/responsibilities matrix (who approves, owns rollbacks).          | Accountability.                                                      | Org chart / access control policy.                     |
|                                | There is a risk assessment/risk register for replacing core binaries.                                 | To identify threats (e.g. broken startup scripts, ownership issues). | Risk register detailing potential issues + mitigation. |
|                                | There is documented policy for backups and restore points.                                            | Ability to recover in case of failure.                               | Backup policy, restore playbooks.                      |
|                                | Audit logging policy exists and is enforced: which events are logged, where logs are kept, retention. | Required for forensic investigation and compliance.                  | Logging configuration; log retention schedule.         |
|                                | Procedures for emergency rollback / disaster recovery are in place and tested.                        | To reduce downtime in critical environments.                         | Test reports; rollback drills.                         |

\| **Security Controls & Integrity** | Cryptographic hashing (e.g. SHA-256) of before/after binaries. | To detect tampering or corruption. | Hash outputs; hash verification protocol. |
\| | Signing / attestation of replaced binaries/providers. | Ensures source integrity. | Digital signatures; public keys; verification logs. |
\| | Secure path handling (against TOCTOU / symlink attacks). | Prevents attackers from subverting replacement. | Code review; static analysis; tests for TOCTOU. |
\| | Ownership, permissions, and modes preserved or enforced. | Prevents privilege escalation or misuse. | `stat` reports; permission tests; policies. |
\| | SELinux/AppArmor, filesystem labels or flags preserved. | Maintains security context. | Label output; policies; post-replacement verification. |
\| | Immutable/append-only flags / hardware protections respected. | Prevents overwriting or tampering. | Check `chattr` / LSM; mount options. |

\| **Atomicity, Backup & Rollback** | Replacement is atomic: staging → rename → fsync(parent). | Avoids broken or missing paths during updates. | Observe filesystem actions; test crash scenarios. |
\| | Backup is taken before any destructive replacement. | To allow safe rollback. | Backup files or backup metadata; backup policy. |
\| | Rollback restores not just content, but metadata (ownership, mode, timestamps, xattrs, caps, labels). | To return system to exact prior state. | Test rollbacks; compare metadata before/after. |
\| | Rollback is idempotent (running it twice is the same as once). | Avoid unexpected changes or glitches. | Repeat rollback tests. |

\| **Preflight / Safety Gates** | All targets checked for immutability, read-only mounts, noexec, etc. | If FS is read-only or immutable, replacement may fail or be dangerous. | Preflight report; mount options. |
\| | Cross-filesystem EXDEV situations handled or disallowed per policy. | Avoid possible broken atomicity. | Detect if staging & target are on different FS; policy file. |
\| | Ownership oracle confirms source proper owner; no world-writable risky files. | Ensures trust in replaced binary. | Ownership records; checks. |
\| | Permissions of parent directory allow safe rename, write. | Avoid failure mid-operation. | `stat` parent; test operations. |

\| **Observability & Audit Logging** | Every replacement emits structured facts: before/after hash, planned vs current kind, owner, timing, error codes. | For traceability. | Audit logs; fact schema. |
\| | Dry-run / plan output identical (modulo redactions) to commit output. | Ensures deterministic behavior. | Compare dry-run vs commit logs. |
\| | Secret masking in logs; no sensitive info exposed. | Avoid leaking secrets. | Review logs; test with dummy secrets. |
\| | Locking observed: only one mutator active, bounded wait, record `lock_wait_ms`. | Prevents concurrent dangerous operations. | Log of lock acquisition; test concurrent apply. |

\| **Health Verification & Testing** | Post-apply smoke tests (ls, cp, mv, rm, ln, stat, readlink, sha256sum, sort, date etc.). | Verifies basic functionality. | Smoke test outputs; failure responses. |
\| | If smoke test fails, automatic rollback triggers. | Prevents degraded system remaining in production. | Logs of smoke, rollback action. |
\| | Tests include edge cases scripts that rely on GNU behaviour. | Ensures compatibility. | Compatibility tests; script suite. |

\| **Package Manager / Ownership / Durability** | If permanent replacement is claimed, package manager ownership is transferred or properly diverted. | Ensures future updates don’t overwrite. | Package metadata; `pacman`, `dpkg-divert`, or equivalent. |
\| | System-wide package conflicts/replaces are set correctly. | Avoids having both `coreutils` and `uutils` causing file conflicts. | Inspect `Provides/Conflicts/Replaces` in package. |
\| | Multi-call or hardlink layout is preserved / recreated. | Certain scripts or tools expect this layout. | Check entrypoints; `ls -l` listings. |

\| **Regulatory / Standard-Specific Controls** | For PCI DSS: log storage, strong access control, monitoring, vulnerability management. | PCI compliance demands these. | PCI audit reports; vulnerability scan results. From sources like Secureframe “Complete PCI DSS Compliance Checklist” ([Secureframe][1]) |
\| | For HIPAA: log access to sensitive info, risk analysis, breach notification procedures, encryption in transit/rest. | Required under HIPAA rules. | HIPAA audit checklist, risk assessment ([The HIPAA Journal][2]) |
\| | For FedRAMP: configuration management, continuous monitoring, incident response, System Security Plan. | To satisfy FedRAMP baseline controls. | Documentation; audit logs ([AuditBoard][3]) |

\| **Change Management & Incident Response** | All binary replacement requests are documented, approved, and reviewed before execution. | Avoid accidental breakage of critical systems. | Change request tickets. |
\| | Incident / failure response procedure defined and rehearsed. | For production readiness. | Incident response policy; test drills. |
\| | Monitoring & alerting on unexpected modifications to critical binaries. | Detect PM or other tampering. | File integrity monitoring; alerts. |

\| **Structural & Operational Controls** | Least privilege: only required privileges used during replacement. | Minimizes attacker impact. | User/group permissions; `sudo` policy. |
\| | Filesystem permissions, ACLs, xattrs/caps correct and secured. | Maintain correct access control. | `stat`, `getfacl`, `getcap`. |
\| | Immutable logs, secure log transfer/storage (s3, WORM, etc). | Prevent logs from being tampered. | Log storage policies; cryptographic integrity. |

\| **Testing & Validation** | Test in staging environment mirroring production before deploy. | Avoid surprises. | Test reports. |
\| | Simulate power-loss or kill during apply to ensure atomicity or safe rollback. | Exposes failure modes. | Test cases. |
\| | Periodic review of smoke suite / regression test with known binaries. | Ensure future changes don’t break compatibility. | Regression test artifacts. |

\| **Legal & Privacy** | Data protection/privacy laws compliance (GDPR, etc.) if user data is involved. | Some binaries handle user data or logs. | Privacy policy; data flow diagrams. |
\| | Licensing compliance for replaced binaries/providers. | Avoid IP/legal issues. | License documents. |

\| **Periodic Review & Audits** | Internal audits of replacement workflow every 6-12 months. | Maintain compliance. | Audit reports. |
\| | Review of facts/logs for unusual error\_ids or degraded operations. | Signal drift or misuse. | Log review schedule; dashboards. |

---

If you like, I can map this checklist into your SPEC\_CHANGE / Switchyard spec so each item ties to a normative requirement (and show which ones you already satisfy vs which need you to add)?

[1]: https://secureframe.com/blog/pci-compliance-checklist?utm_source=chatgpt.com "Complete PCI DSS Compliance Checklist: Navigate the 12 ..."
[2]: https://www.hipaajournal.com/hipaa-audit-checklist/?utm_source=chatgpt.com "HIPAA Audit Checklist - 2025 Update"
[3]: https://auditboard.com/blog/fedramp-checklist?utm_source=chatgpt.com "Navigate FedRAMP: A step-by-step checklist"
