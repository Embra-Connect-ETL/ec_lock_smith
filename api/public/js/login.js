const { createApp } = Vue;

function setCookie(cname, cvalue, exdays) {
    const d = new Date();
    d.setTime(d.getTime() + (exdays * 24 * 60 * 60 * 1000));
    let expires = "expires=" + d.toUTCString();
    document.cookie = cname + "=" + cvalue + ";" + expires + ";path=/";
}

function getCookie(cname) {
    let name = cname + "=";
    let ca = document.cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(name) == 0) {
            return c.substring(name.length, c.length);
        }
    }
    return "";
}

createApp({
    data() {
        return {
            email: "",
            password: "",
            loading: false
        };
    },
    methods: {
        async handleLogin() {
            if (!this.email || !this.password) {
                this.displayToaster("Please enter email and password", "red");
                return;
            }

            // Prevent multiple requests
            if (this.loading) return;
            this.loading = true;

            try {
                const response = await fetch(`${API_BASE_URL}/login`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ email: this.email, password: this.password }),
                });

                const data = await response.json();

                if (data.status !== 200) {
                    this.displayToaster(data.message || "Invalid credentials", "red");
                    return;
                }

                // Store credentials and redirect
                sessionStorage.setItem("ec_id", this.email);
                setCookie("ec_id", this.email, 1);
                this.displayToaster("Login successful, redirecting...");

                setTimeout(() => {
                    window.location.href = "./console.html";
                }, 100);

            } catch (error) {
                this.displayToaster(error.message || "Something went wrong, please try again", "red");
                console.error("[Error]::[Auth] -> ", error);
            } finally {
                this.loading = false;
            }
        },
        displayToaster(message, backgroundColor = "#ffa07a") {
            Toastify({
                text: message,
                duration: 3000,
                close: false,
                gravity: "top",
                position: "right",
                style: {
                    background: backgroundColor,
                    color: "#ffffff",
                    borderRadius: "24px",
                    fontWeight: "600",
                    fontSize: "14px",
                    letterSpacing: "1.4px",
                    textTransform: "capitalize",
                    boxShadow: "0 1rem 1rem 0 rgba(0, 0, 0, .05)"
                }
            }).showToast();
        }
    }
}).mount("#app");
