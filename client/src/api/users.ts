import { apiFetch } from "./client"
import type { Role, User } from "./types"

/** GET /users — list all users (Admin only). */
export function listUsers(): Promise<User[]> {
  return apiFetch<User[]>("/users")
}

export interface CreateUserInput {
  username: string
  password: string
  role: Role
}

/** POST /users — create a user (Admin only). */
export function createUser(input: CreateUserInput): Promise<User> {
  return apiFetch<User>("/users", { method: "POST", body: input })
}

export interface UpdateUserInput {
  role?: Role
  password?: string
}

/** PATCH /users/:id — update role or reset password (Admin only). */
export function updateUser(userId: string, input: UpdateUserInput): Promise<User> {
  return apiFetch<User>(`/users/${encodeURIComponent(userId)}`, {
    method: "PATCH",
    body: input,
  })
}

/** DELETE /users/:id — deactivate user (soft delete, Admin only). Returns 204. */
export function deleteUser(userId: string): Promise<void> {
  return apiFetch(`/users/${encodeURIComponent(userId)}`, {
    method: "DELETE",
  })
}
