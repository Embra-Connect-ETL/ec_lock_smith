const { createApp, reactive, ref } = Vue;

createApp({
    setup() {
        const formData = reactive({
            email: '',
            password: ''
        });

        const isLoading = ref(false); // Track request state

        const displayToaster = (message, backgroundColor = "#ffa07a") => {
            Toastify({
                text: message,
                duration: 3000,
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
        };

        const registerUser = async () => {
            if (!formData.email || !formData.password) {
                displayToaster("Email and password are required!");
                return;
            }

            if (isLoading.value) return; // Prevent multiple clicks
            isLoading.value = true; // Disable form interaction

            try {
                const response = await fetch(`${API_BASE_URL}/setup`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ email: formData.email, password: formData.password })
                });

                const data = await response.json();

                if (data.status !== 200) {
                    displayToaster(data.message || "Registration failed!", "red");
                } else {
                    displayToaster("Registration successful!");

                    setTimeout(() => {
                        window.location.href = "./login.html";
                    }, 2000);
                }
            } catch (error) {
                displayToaster("Something went wrong, please try again", "red");
                console.error("[Error]::[Auth] -> ", error);
            } finally {
                isLoading.value = false; // Re-enable form interaction
            }
        };

        return { formData, registerUser, isLoading };
    }
}).mount("#app");
