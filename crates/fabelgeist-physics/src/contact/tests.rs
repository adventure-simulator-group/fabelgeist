use super::*;
use crate::collider::COLLIDER_BYTES;

async fn bytes(collision: &Collisions, context: &WgpuContext) -> Vec<u8> {
    collision.collider_buffer.read(context).await.unwrap()
}

#[tokio::test]
async fn growth_and_empty_replacement_keep_native_packing_and_retained_capacity() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let mut collision = Collisions::new(&context, &cache).unwrap();
    assert_eq!(
        u64::from(collision.collider_buffer.size),
        (16 * COLLIDER_BYTES) as u64
    );
    let colliders: Vec<_> = (0..17).map(|i| Collider::ground(i as f32)).collect();
    let expected = bytemuck::cast_slice::<f32, u8>(&pack_colliders(&colliders)).to_vec();
    collision.set_colliders(&context, colliders).unwrap();
    let uploaded = bytes(&collision, &context).await;
    assert_eq!(uploaded.len(), 32 * COLLIDER_BYTES);
    assert_eq!(&uploaded[..expected.len()], &expected);
    collision.set_colliders(&context, vec![]).unwrap();
    assert!(collision.colliders().is_empty());
    assert_eq!(bytes(&collision, &context).await, uploaded);
    collision
        .set_colliders(&context, vec![Collider::ground(1.25)])
        .unwrap();
    let smaller = bytes(&collision, &context).await;
    assert_eq!(smaller.len(), uploaded.len());
    assert_eq!(&smaller[COLLIDER_BYTES..], &uploaded[COLLIDER_BYTES..]);
}

#[tokio::test]
async fn rejected_update_keeps_host_list_and_uploaded_words() {
    let context = WgpuContext::new().await.unwrap();
    let cache = KernelCache::new();
    let mut collision = Collisions::new(&context, &cache).unwrap();
    collision
        .set_colliders(&context, vec![Collider::ground(0.5)])
        .unwrap();
    let before = bytes(&collision, &context).await;
    let error = collision
        .update_colliders(&context, &[Collider::ground(1.0), Collider::ground(2.0)])
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "Collisions::update_colliders: holds 1 colliders, given 2; use `set_colliders` to change the count"
    );
    assert_eq!(collision.colliders().len(), 1);
    assert_eq!(bytes(&collision, &context).await, before);
    let held = bytemuck::cast_slice::<f32, u8>(&pack_colliders(collision.colliders())).to_vec();
    assert_eq!(&before[..held.len()], &held);
}
