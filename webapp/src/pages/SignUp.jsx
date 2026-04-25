import { useState, useRef } from "react";
import { Link, useNavigate } from "react-router-dom";
import { motion, AnimatePresence } from "framer-motion";
import { ArrowRight, ArrowLeft, Shield, Car, Star } from "lucide-react";
import { useAuth } from "../hooks/useAuth";
import { signUp } from "../utils/api";
import api from "../utils/api";
import Logo from "../components/Logo";

export default function SignUp() {
  const [form, setForm] = useState({
    full_name: "",
    email: "",
    password: "",
    country: "Nigeria",
  });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [step, setStep] = useState("details");
  const [code, setCode] = useState(["", "", "", ""]);
  const [resending, setResending] = useState(false);
  const [maskedEmail, setMaskedEmail] = useState("");
  const codeRefs = useRef([]);
  const { login } = useAuth();
  const navigate = useNavigate();

  const set = (key) => (e) => setForm({ ...form, [key]: e.target.value });

  const maskEmail = (email) => {
    const [local, domain] = email.split("@");
    if (local.length <= 2) return email;
    return local[0] + "***" + local[local.length - 1] + "@" + domain;
  };

  const handleSendCode = async (e) => {
    e.preventDefault();
    setError("");
    if (!form.full_name.trim()) {
      setError("Please enter your name");
      return;
    }
    if (!form.email.trim()) {
      setError("Please enter your email");
      return;
    }
    if (form.password.length < 6) {
      setError("Password must be at least 6 characters");
      return;
    }
    setLoading(true);
    try {
      await api.post("/auth/send-code", {
        email: form.email.trim().toLowerCase(),
      });
      setMaskedEmail(maskEmail(form.email.trim()));
      setStep("verify");
    } catch (err) {
      setError(err.response?.data?.error || "Failed to send verification code");
    }
    setLoading(false);
  };

  const handleCodeChange = (index, value) => {
    if (value.length > 1) value = value[value.length - 1];
    const newCode = [...code];
    newCode[index] = value;
    setCode(newCode);
    if (value && index < 3) codeRefs.current[index + 1]?.focus();
    if (value && index === 3 && newCode.every((d) => d))
      handleVerify(newCode.join(""));
  };

  const handleCodeKeyDown = (index, e) => {
    if (e.key === "Backspace" && !code[index] && index > 0)
      codeRefs.current[index - 1]?.focus();
  };

  const handleVerify = async (codeStr) => {
    const fullCode = codeStr || code.join("");
    if (fullCode.length !== 4) {
      setError("Please enter the 4-digit code");
      return;
    }
    setError("");
    setLoading(true);
    try {
      await api.post("/auth/verify-code", {
        email: form.email.trim().toLowerCase(),
        code: fullCode,
      });
      const res = await signUp({ ...form, role: "Renter" });
      login(res.data.token, res.data.refresh_token, res.data.user);
      navigate("/");
    } catch (err) {
      setError(err.response?.data?.error || "Verification failed");
      setCode(["", "", "", ""]);
      codeRefs.current[0]?.focus();
    }
    setLoading(false);
  };

  const handleResend = async () => {
    setResending(true);
    setError("");
    try {
      await api.post("/auth/send-code", {
        email: form.email.trim().toLowerCase(),
      });
    } catch {
      setError("Failed to resend code");
    }
    setResending(false);
  };

  return (
    <div style={{ minHeight: "100vh", display: "flex" }}>
      {/* Left — visual panel */}
      <div
        style={{
          flex: 1,
          position: "relative",
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
          justifyContent: "flex-end",
          padding: "60px 48px",
          background:
            "linear-gradient(160deg, #0D1F13 0%, #060A06 50%, #0A0A0A 100%)",
        }}
      >
        {/* Ambient glow */}
        <div
          style={{
            position: "absolute",
            top: "10%",
            left: "20%",
            width: 500,
            height: 500,
            borderRadius: "50%",
            background:
              "radial-gradient(circle, rgba(34,197,94,0.08) 0%, transparent 70%)",
            pointerEvents: "none",
            filter: "blur(60px)",
          }}
        />
        <div
          style={{
            position: "absolute",
            bottom: "15%",
            right: "10%",
            width: 400,
            height: 400,
            borderRadius: "50%",
            background:
              "radial-gradient(circle, rgba(34,197,94,0.05) 0%, transparent 70%)",
            pointerEvents: "none",
            filter: "blur(80px)",
          }}
        />
        {/* Grid pattern */}
        <div
          style={{
            position: "absolute",
            inset: 0,
            opacity: 0.03,
            backgroundImage:
              "linear-gradient(rgba(255,255,255,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.5) 1px, transparent 1px)",
            backgroundSize: "80px 80px",
            pointerEvents: "none",
          }}
        />

        {/* Animated car */}
        <SpeedingCar />

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          style={{ position: "relative", zIndex: 1 }}
        >
          <Link
            to="/"
            style={{
              textDecoration: "none",
              display: "inline-block",
              marginBottom: 48,
            }}
          >
            <Logo variant="full" size={32} />
          </Link>
          <h2
            style={{
              fontSize: 52,
              fontWeight: 900,
              letterSpacing: -2,
              lineHeight: 1.05,
              marginBottom: 16,
              maxWidth: 440,
            }}
          >
            Start your
            <br />
            journey.
          </h2>
          <p
            style={{
              color: "rgba(255,255,255,0.4)",
              fontSize: 16,
              lineHeight: 1.6,
              maxWidth: 380,
            }}
          >
            Join thousands of Nigerians who rent smarter with Qent.
          </p>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 16,
              marginTop: 48,
              paddingTop: 28,
              borderTop: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            {[
              [
                Shield,
                "Verified hosts & renters",
                "Every user goes through identity verification",
              ],
              [Car, "Premium vehicles", "From everyday rides to luxury cars"],
              [
                Star,
                "Rated community",
                "Two-way reviews keep everyone accountable",
              ],
            ].map(([Icon, title, sub]) => (
              <div
                key={title}
                style={{ display: "flex", gap: 14, alignItems: "flex-start" }}
              >
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 10,
                    flexShrink: 0,
                    background: "rgba(34,197,94,0.08)",
                    border: "1px solid rgba(34,197,94,0.12)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                  }}
                >
                  <Icon size={16} color="#22C55E" />
                </div>
                <div>
                  <div
                    style={{ fontSize: 14, fontWeight: 600, marginBottom: 2 }}
                  >
                    {title}
                  </div>
                  <div
                    style={{ fontSize: 12, color: "rgba(255,255,255,0.35)" }}
                  >
                    {sub}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>

      {/* Right — form */}
      <div
        style={{
          width: 520,
          flexShrink: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "80px 48px",
          background: "#0A0A0A",
          borderLeft: "1px solid rgba(255,255,255,0.06)",
        }}
      >
        <div style={{ width: "100%", maxWidth: 380 }}>
          <AnimatePresence mode="wait">
            {step === "details" ? (
              <motion.div
                key="details"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                transition={{ duration: 0.4 }}
              >
                <h1
                  style={{
                    fontSize: 30,
                    fontWeight: 800,
                    letterSpacing: -0.5,
                    marginBottom: 6,
                  }}
                >
                  Create account
                </h1>
                <p
                  style={{
                    color: "rgba(255,255,255,0.4)",
                    fontSize: 14,
                    marginBottom: 36,
                  }}
                >
                  Join Qent and start renting
                </p>

                {error && (
                  <motion.div
                    initial={{ opacity: 0, y: -6 }}
                    animate={{ opacity: 1, y: 0 }}
                    style={errorStyle}
                  >
                    {String(error)}
                  </motion.div>
                )}

                <form onSubmit={handleSendCode}>
                  <Input
                    label="Full Name"
                    value={form.full_name}
                    onChange={set("full_name")}
                    placeholder="Your full name"
                  />
                  <Input
                    label="Email"
                    type="email"
                    value={form.email}
                    onChange={set("email")}
                    placeholder="you@example.com"
                  />
                  <Input
                    label="Password"
                    type="password"
                    value={form.password}
                    onChange={set("password")}
                    placeholder="At least 6 characters"
                  />

                  <button
                    type="submit"
                    disabled={loading}
                    style={{
                      width: "100%",
                      padding: 16,
                      marginTop: 8,
                      background: loading ? "rgba(34,197,94,0.5)" : "#22C55E",
                      color: "#0A0A0A",
                      border: "none",
                      borderRadius: 14,
                      fontSize: 15,
                      fontWeight: 700,
                      cursor: loading ? "not-allowed" : "pointer",
                      fontFamily: "inherit",
                      transition: "all 0.2s",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      gap: 8,
                    }}
                  >
                    {loading ? (
                      "Sending code..."
                    ) : (
                      <>
                        Continue <ArrowRight size={16} />
                      </>
                    )}
                  </button>
                </form>

                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 12,
                    margin: "28px 0",
                  }}
                >
                  <div
                    style={{
                      flex: 1,
                      height: 1,
                      background: "rgba(255,255,255,0.06)",
                    }}
                  />
                  <span
                    style={{
                      fontSize: 12,
                      color: "rgba(255,255,255,0.25)",
                      fontWeight: 500,
                    }}
                  >
                    OR
                  </span>
                  <div
                    style={{
                      flex: 1,
                      height: 1,
                      background: "rgba(255,255,255,0.06)",
                    }}
                  />
                </div>

                <Link
                  to="/login"
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    width: "100%",
                    padding: 16,
                    borderRadius: 14,
                    background: "rgba(255,255,255,0.04)",
                    border: "1px solid rgba(255,255,255,0.08)",
                    color: "white",
                    fontSize: 15,
                    fontWeight: 600,
                    textDecoration: "none",
                  }}
                >
                  Sign in instead
                </Link>
              </motion.div>
            ) : (
              <motion.div
                key="verify"
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -20 }}
                transition={{ duration: 0.4 }}
              >
                <button
                  onClick={() => {
                    setStep("details");
                    setError("");
                    setCode(["", "", "", ""]);
                  }}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                    background: "none",
                    border: "none",
                    color: "rgba(255,255,255,0.5)",
                    fontSize: 14,
                    cursor: "pointer",
                    fontFamily: "inherit",
                    marginBottom: 24,
                    padding: 0,
                  }}
                >
                  <ArrowLeft size={16} /> Back
                </button>

                <h1
                  style={{
                    fontSize: 30,
                    fontWeight: 800,
                    letterSpacing: -0.5,
                    marginBottom: 6,
                  }}
                >
                  Verify email
                </h1>
                <p
                  style={{
                    color: "rgba(255,255,255,0.4)",
                    fontSize: 14,
                    marginBottom: 36,
                  }}
                >
                  Enter the 4-digit code sent to {maskedEmail}
                </p>

                {error && (
                  <motion.div
                    initial={{ opacity: 0, y: -6 }}
                    animate={{ opacity: 1, y: 0 }}
                    style={errorStyle}
                  >
                    {String(error)}
                  </motion.div>
                )}

                <div
                  style={{
                    display: "flex",
                    gap: 14,
                    justifyContent: "center",
                    marginBottom: 36,
                  }}
                >
                  {[0, 1, 2, 3].map((i) => (
                    <input
                      key={i}
                      ref={(el) => (codeRefs.current[i] = el)}
                      type="text"
                      inputMode="numeric"
                      maxLength={1}
                      value={code[i]}
                      onChange={(e) =>
                        handleCodeChange(i, e.target.value.replace(/\D/g, ""))
                      }
                      onKeyDown={(e) => handleCodeKeyDown(i, e)}
                      autoFocus={i === 0}
                      style={{
                        width: 64,
                        height: 72,
                        textAlign: "center",
                        fontSize: 28,
                        fontWeight: 800,
                        background: code[i]
                          ? "rgba(34,197,94,0.05)"
                          : "rgba(255,255,255,0.03)",
                        border: `2px solid ${
                          code[i]
                            ? "rgba(34,197,94,0.3)"
                            : "rgba(255,255,255,0.08)"
                        }`,
                        borderRadius: 16,
                        color: "white",
                        outline: "none",
                        fontFamily: "inherit",
                        transition: "all 0.2s",
                      }}
                      onFocus={(e) =>
                        (e.target.style.borderColor = "rgba(34,197,94,0.5)")
                      }
                      onBlur={(e) =>
                        (e.target.style.borderColor = code[i]
                          ? "rgba(34,197,94,0.3)"
                          : "rgba(255,255,255,0.08)")
                      }
                    />
                  ))}
                </div>

                <button
                  onClick={() => handleVerify()}
                  disabled={loading || code.some((d) => !d)}
                  style={{
                    width: "100%",
                    padding: 16,
                    background:
                      loading || code.some((d) => !d)
                        ? "rgba(34,197,94,0.3)"
                        : "#22C55E",
                    color: "#0A0A0A",
                    border: "none",
                    borderRadius: 14,
                    fontSize: 15,
                    fontWeight: 700,
                    cursor:
                      loading || code.some((d) => !d)
                        ? "not-allowed"
                        : "pointer",
                    fontFamily: "inherit",
                    transition: "all 0.2s",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 8,
                  }}
                >
                  {loading ? (
                    "Verifying..."
                  ) : (
                    <>
                      Verify & Create Account <ArrowRight size={16} />
                    </>
                  )}
                </button>

                <p
                  style={{
                    textAlign: "center",
                    marginTop: 24,
                    color: "rgba(255,255,255,0.3)",
                    fontSize: 13,
                  }}
                >
                  Didn't get the code?{" "}
                  <button
                    onClick={handleResend}
                    disabled={resending}
                    style={{
                      background: "none",
                      border: "none",
                      color: "#22C55E",
                      fontWeight: 600,
                      cursor: "pointer",
                      fontFamily: "inherit",
                      fontSize: 13,
                    }}
                  >
                    {resending ? "Sending..." : "Resend"}
                  </button>
                </p>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>

      <style>{`
        @media (max-width: 900px) {
          div[style*="flex: 1"] { display: none !important; }
          div[style*="width: 520"] { width: 100% !important; border: none !important; }
        }
      `}</style>
    </div>
  );
}

function Input({ label, type = "text", value, onChange, placeholder }) {
  return (
    <div style={{ marginBottom: 18 }}>
      <label
        style={{
          fontSize: 13,
          fontWeight: 600,
          color: "rgba(255,255,255,0.5)",
          display: "block",
          marginBottom: 8,
        }}
      >
        {label}
      </label>
      <input
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        style={{
          width: "100%",
          padding: "15px 16px",
          border: "1px solid rgba(255,255,255,0.08)",
          background: "rgba(255,255,255,0.03)",
          borderRadius: 14,
          fontSize: 14,
          color: "white",
          outline: "none",
          fontFamily: "inherit",
          boxSizing: "border-box",
          transition: "all 0.2s",
        }}
        onFocus={(e) => {
          e.target.style.borderColor = "rgba(34,197,94,0.4)";
          e.target.style.background = "rgba(34,197,94,0.03)";
        }}
        onBlur={(e) => {
          e.target.style.borderColor = "rgba(255,255,255,0.08)";
          e.target.style.background = "rgba(255,255,255,0.03)";
        }}
      />
    </div>
  );
}

function SpeedingCar() {
  return (
    <div
      style={{
        position: "absolute",
        top: "25%",
        left: 0,
        right: 0,
        height: 200,
        pointerEvents: "none",
        overflow: "hidden",
      }}
    >
      <motion.div
        style={{
          position: "absolute",
          bottom: 68,
          left: 0,
          right: 0,
          height: 1,
          background:
            "linear-gradient(90deg, transparent 0%, rgba(34,197,94,0.15) 30%, rgba(34,197,94,0.15) 70%, transparent 100%)",
        }}
      />
      {[0, 1, 2, 3, 4, 5, 6].map((i) => (
        <motion.div
          key={i}
          initial={{ x: "60%", opacity: 0 }}
          animate={{ x: "-120%", opacity: [0, 0.8, 0] }}
          transition={{
            duration: 0.6,
            delay: i * 0.12,
            repeat: Infinity,
            repeatDelay: 0.1,
            ease: "linear",
          }}
          style={{
            position: "absolute",
            bottom: 74 + (i - 3) * 10,
            width: 80 + Math.random() * 60,
            height: 1.5,
            borderRadius: 2,
            background: `linear-gradient(90deg, transparent, rgba(34,197,94,${
              0.2 + i * 0.06
            }))`,
          }}
        />
      ))}
      <motion.div
        initial={{ x: "-40%", opacity: 0 }}
        animate={{ x: "25%", opacity: 1 }}
        transition={{ duration: 0.8, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        style={{ position: "absolute", bottom: 52 }}
      >
        <svg width="220" height="80" viewBox="0 0 220 80" fill="none">
          <path
            d="M35 50 L55 22 L95 12 L155 12 L185 28 L210 38 L210 55 L35 55 Z"
            fill="rgba(34,197,94,0.12)"
            stroke="rgba(34,197,94,0.3)"
            strokeWidth="1"
          />
          <path
            d="M60 24 L90 14 L130 14 L130 34 L60 34 Z"
            fill="rgba(34,197,94,0.06)"
            stroke="rgba(34,197,94,0.15)"
            strokeWidth="0.5"
          />
          <path
            d="M135 14 L155 14 L175 30 L135 34 Z"
            fill="rgba(34,197,94,0.06)"
            stroke="rgba(34,197,94,0.15)"
            strokeWidth="0.5"
          />
          <circle cx="205" cy="42" r="8" fill="rgba(34,197,94,0.2)" />
          <circle cx="205" cy="42" r="3" fill="rgba(34,197,94,0.5)" />
          <circle cx="38" cy="48" r="3" fill="rgba(239,68,68,0.4)" />
          <circle
            cx="170"
            cy="58"
            r="12"
            fill="#0A0A0A"
            stroke="rgba(255,255,255,0.15)"
            strokeWidth="1.5"
          />
          <circle cx="170" cy="58" r="5" fill="rgba(255,255,255,0.08)" />
          <circle
            cx="70"
            cy="58"
            r="12"
            fill="#0A0A0A"
            stroke="rgba(255,255,255,0.15)"
            strokeWidth="1.5"
          />
          <circle cx="70" cy="58" r="5" fill="rgba(255,255,255,0.08)" />
          <line
            x1="45"
            y1="55"
            x2="200"
            y2="55"
            stroke="rgba(34,197,94,0.2)"
            strokeWidth="1"
          />
        </svg>
        <motion.div
          animate={{ opacity: [0.3, 0.6, 0.3] }}
          transition={{ duration: 2, repeat: Infinity }}
          style={{
            position: "absolute",
            right: -80,
            top: 18,
            width: 100,
            height: 30,
            background:
              "linear-gradient(90deg, rgba(34,197,94,0.15), transparent)",
            borderRadius: "0 50% 50% 0",
            filter: "blur(8px)",
          }}
        />
      </motion.div>
      {[0, 1, 2, 3, 4].map((i) => (
        <motion.div
          key={`p${i}`}
          animate={{ x: [300, -100], opacity: [0, 0.7, 0] }}
          transition={{
            duration: 0.5,
            delay: i * 0.2 + 0.5,
            repeat: Infinity,
            repeatDelay: 0.3,
            ease: "linear",
          }}
          style={{
            position: "absolute",
            bottom: 66,
            width: 3,
            height: 3,
            borderRadius: "50%",
            background: "rgba(34,197,94,0.3)",
          }}
        />
      ))}
    </div>
  );
}

const errorStyle = {
  background: "rgba(239,68,68,0.08)",
  color: "#EF4444",
  padding: "12px 16px",
  borderRadius: 14,
  fontSize: 13,
  fontWeight: 500,
  marginBottom: 20,
  border: "1px solid rgba(239,68,68,0.15)",
};
